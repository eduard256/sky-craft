// Game state and main game loop. Central hub that ties everything together.
// Runs at 20 TPS, processes all game logic each tick.

use std::sync::{Arc, RwLock, atomic::{AtomicU32, AtomicU64, Ordering}};
use dashmap::DashMap;

use skycraft_protocol::packets::*;
use skycraft_protocol::types::*;
use skycraft_protocol::MS_PER_TICK;

use crate::config::ServerConfig;
use crate::world::World;
use crate::player::Player;
use crate::entity::{Entity, EntityKind, AIState, MobCategory};
use crate::physics;

/// Central game state shared across all connection handlers and the game loop.
pub struct GameState {
    pub config: Arc<ServerConfig>,
    pub world: Arc<World>,

    // Players indexed by entity_id
    players: DashMap<EntityId, RwLock<Player>>,
    // Non-player entities indexed by entity_id
    entities: DashMap<EntityId, RwLock<Entity>>,
    // Nickname -> entity_id for quick lookup
    nickname_map: DashMap<String, EntityId>,

    // ID counters
    next_entity_id: AtomicU32,

    // World time
    world_age: AtomicU64,
    time_of_day: AtomicU32,

    // Weather
    weather: RwLock<Weather>,
    weather_ticks_remaining: AtomicU32,

    // Wind (per-tick updated)
    wind: RwLock<WindState>,

    // Keep-alive counter
    keep_alive_counter: AtomicU64,
}

impl GameState {
    pub fn new(config: Arc<ServerConfig>, world: Arc<World>) -> Self {
        Self {
            config,
            world,
            players: DashMap::new(),
            entities: DashMap::new(),
            nickname_map: DashMap::new(),
            next_entity_id: AtomicU32::new(1),
            world_age: AtomicU64::new(0),
            time_of_day: AtomicU32::new(0),
            weather: RwLock::new(Weather::Clear),
            weather_ticks_remaining: AtomicU32::new(12000),
            wind: RwLock::new(WindState { direction: 0.0, strength: 0.0, gusting: false }),
            keep_alive_counter: AtomicU64::new(0),
        }
    }

    // ── Player Management ───────────────────────────────────────────────────

    /// Add a player to the game. Returns assigned entity_id.
    pub fn add_player(&self, mut player: Player) -> EntityId {
        let id = self.next_entity_id.fetch_add(1, Ordering::Relaxed);
        player.entity_id = id;
        let nickname = player.nickname.clone();
        self.players.insert(id, RwLock::new(player));
        self.nickname_map.insert(nickname, id);
        id
    }

    /// Remove a player from the game.
    pub fn remove_player(&self, entity_id: EntityId) {
        if let Some((_, player_lock)) = self.players.remove(&entity_id) {
            if let Ok(player) = player_lock.read() {
                self.nickname_map.remove(&player.nickname);
            }
        }
    }

    /// Check if a player with this nickname is online.
    pub fn is_player_online(&self, nickname: &str) -> bool {
        self.nickname_map.contains_key(nickname)
    }

    /// Get current number of connected players.
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Execute a closure with read access to a player.
    pub fn with_player<F, R>(&self, entity_id: EntityId, f: F) -> Option<R>
    where F: FnOnce(&Player) -> R {
        self.players.get(&entity_id).and_then(|entry| {
            entry.value().read().ok().map(|p| f(&*p))
        })
    }

    /// Execute a closure with write access to a player.
    pub fn with_player_mut<F, R>(&self, entity_id: EntityId, f: F) -> Option<R>
    where F: FnOnce(&mut Player) -> R {
        self.players.get(&entity_id).and_then(|entry| {
            entry.value().write().ok().map(|mut p| f(&mut *p))
        })
    }

    // ── Packet Distribution ─────────────────────────────────────────────────

    /// Broadcast a packet to all players except `exclude`.
    pub fn broadcast_packet(&self, packet: &ServerPacket, exclude: Option<EntityId>) {
        for entry in self.players.iter() {
            let eid = *entry.key();
            if Some(eid) == exclude {
                continue;
            }
            if let Ok(player) = entry.value().read() {
                player.send_packet(packet.clone());
            }
        }
    }

    /// Send a packet to a specific player.
    pub fn send_to_player(&self, entity_id: EntityId, packet: ServerPacket) {
        if let Some(entry) = self.players.get(&entity_id) {
            if let Ok(player) = entry.value().read() {
                player.send_packet(packet);
            }
        }
    }

    /// Pop the next outbound packet for a player (called by network layer).
    pub fn pop_outbound_packet(&self, entity_id: EntityId) -> Option<ServerPacket> {
        self.players.get(&entity_id).and_then(|entry| {
            entry.value().read().ok().and_then(|p| p.pop_packet())
        })
    }

    // ── Time ────────────────────────────────────────────────────────────────

    pub fn world_age(&self) -> u64 {
        self.world_age.load(Ordering::Relaxed)
    }

    pub fn time_of_day(&self) -> u32 {
        self.time_of_day.load(Ordering::Relaxed)
    }

    pub fn current_weather(&self) -> Weather {
        *self.weather.read().unwrap_or_else(|e| e.into_inner())
    }

    // ── Packet Handlers ─────────────────────────────────────────────────────

    pub fn update_player_position(&self, entity_id: EntityId, pos: EntityPos, on_ground: bool) {
        let old_pos = self.with_player(entity_id, |p| p.position);

        self.with_player_mut(entity_id, |player| {
            // Track distance for stats
            let dist = player.position.distance_to(&pos);
            if dist < 100.0 { // ignore teleport-sized jumps
                player.statistics.distance_walked += dist;
                // Exhaustion from walking/sprinting
                if player.is_sprinting {
                    player.add_exhaustion(dist as f32 * 0.1);
                }
            }

            player.position = pos;
            player.on_ground = on_ground;

            // Update ring
            let new_ring = player.calculate_ring();
            if new_ring != player.current_ring {
                let old_ring = player.current_ring;
                player.current_ring = new_ring;
                if new_ring > player.statistics.highest_ring {
                    player.statistics.highest_ring = new_ring;
                }
            }
        });

        // Send new chunks if player moved to a new chunk column
        if let Some(old) = old_pos {
            let old_cx = (old.x / 16.0).floor() as i32;
            let old_cz = (old.z / 16.0).floor() as i32;
            let new_cx = (pos.x / 16.0).floor() as i32;
            let new_cz = (pos.z / 16.0).floor() as i32;

            if old_cx != new_cx || old_cz != new_cz {
                let vd = self.config.view_distance as i32;
                let mut packets = Vec::new();
                for cx in (new_cx - vd)..=(new_cx + vd) {
                    for cz in (new_cz - vd)..=(new_cz + vd) {
                        if (cx - old_cx).abs() <= vd && (cz - old_cz).abs() <= vd {
                            continue;
                        }
                        for cy in -4..20 {
                            let chunk_pos = ChunkPos::new(cx, cy, cz);
                            let section = self.world.get_or_generate_chunk(chunk_pos);
                            if !section.is_empty() {
                                packets.push(ServerPacket::ChunkData(S2CChunkData { chunk_pos, section }));
                            }
                        }
                    }
                }
                self.with_player(entity_id, |p| {
                    for pkt in packets { p.send_packet(pkt); }
                });
            }
        }

        // Broadcast movement to other players
        if let Some(old) = old_pos {
            let dx = ((pos.x - old.x) * 4096.0) as i16;
            let dy = ((pos.y - old.y) * 4096.0) as i16;
            let dz = ((pos.z - old.z) * 4096.0) as i16;

            if dx != 0 || dy != 0 || dz != 0 {
                let move_packet = ServerPacket::EntityMove(S2CEntityMove {
                    entity_id,
                    dx, dy, dz,
                    on_ground,
                });
                self.broadcast_packet(&move_packet, Some(entity_id));
            }
        }

        // Check void damage
        if pos.y < physics::VOID_DAMAGE_Y {
            self.apply_void_damage(entity_id, pos);
        }
    }

    pub fn update_player_look(&self, entity_id: EntityId, yaw: f32, pitch: f32) {
        self.with_player_mut(entity_id, |player| {
            player.rotation = Rotation { yaw, pitch };
        });

        let look_packet = ServerPacket::EntityLook(S2CEntityLook {
            entity_id,
            yaw, pitch,
            on_ground: true,
        });
        self.broadcast_packet(&look_packet, Some(entity_id));
    }

    pub fn handle_block_dig(&self, entity_id: EntityId, dig: C2SBlockDig) {
        match dig.action {
            DiggingAction::Start => {
                // Check placement lock debuff
                let has_lock = self.with_player(entity_id, |p| {
                    p.has_debuff(&MobDebuff::MiningLock(0))
                }).unwrap_or(false);

                if has_lock {
                    return; // silently ignore
                }

                self.with_player_mut(entity_id, |p| {
                    p.digging_block = Some(dig.position);
                    p.digging_progress = 0.0;
                });
            }
            DiggingAction::Cancel => {
                self.with_player_mut(entity_id, |p| {
                    p.digging_block = None;
                    p.digging_progress = 0.0;
                });
            }
            DiggingAction::Finish => {
                // Creative mode: instant break
                let game_mode = self.with_player(entity_id, |p| p.game_mode);
                if game_mode == Some(GameMode::Creative) {
                    self.break_block(entity_id, dig.position);
                    return;
                }

                // Survival: verify digging was in progress for correct block
                let valid = self.with_player(entity_id, |p| {
                    p.digging_block == Some(dig.position)
                }).unwrap_or(false);

                if valid {
                    self.break_block(entity_id, dig.position);
                }
            }
        }
    }

    pub fn handle_block_place(&self, entity_id: EntityId, place: C2SBlockPlace) {
        // Check placement lock debuff
        let has_lock = self.with_player(entity_id, |p| {
            p.has_debuff(&MobDebuff::PlacementLock(0))
        }).unwrap_or(false);

        if has_lock {
            return;
        }

        // Calculate target position based on face
        let target = match place.face {
            BlockFace::Bottom => BlockPos::new(place.position.x, place.position.y - 1, place.position.z),
            BlockFace::Top    => BlockPos::new(place.position.x, place.position.y + 1, place.position.z),
            BlockFace::North  => BlockPos::new(place.position.x, place.position.y, place.position.z - 1),
            BlockFace::South  => BlockPos::new(place.position.x, place.position.y, place.position.z + 1),
            BlockFace::West   => BlockPos::new(place.position.x - 1, place.position.y, place.position.z),
            BlockFace::East   => BlockPos::new(place.position.x + 1, place.position.y, place.position.z),
        };

        // Check target is air
        if self.world.get_block(target) != 0 {
            return;
        }

        // Check clicked block is solid (MC rule: must place against existing block)
        if self.world.get_block(place.position) == 0 {
            return;
        }

        // Get held item and determine block to place
        let block_to_place = self.with_player(entity_id, |p| {
            p.held_item().as_ref().map(|item| item.item_id)
        }).flatten();

        if let Some(item_id) = block_to_place {
            // Simplified: item_id roughly maps to block state for basic blocks
            // Full mapping would use items.json -> blocks.json lookup
            let block_state = item_id; // simplified for v0.0.1

            self.world.set_block(target, block_state);

            // Broadcast block change
            let change = ServerPacket::BlockChange(S2CBlockChange {
                position: target,
                block_state,
            });
            self.broadcast_packet(&change, None);

            // Consume item from inventory
            self.with_player_mut(entity_id, |p| {
                let slot = p.held_slot as usize;
                if let Some(ref mut stack) = p.inventory[slot] {
                    stack.count -= 1;
                    if stack.count == 0 {
                        p.inventory[slot] = None;
                    }
                }
                p.statistics.blocks_placed += 1;
                p.add_exhaustion(0.005);
            });
        }
    }

    pub fn handle_chat(&self, entity_id: EntityId, message: String) {
        if message.len() > skycraft_protocol::MAX_CHAT_LENGTH {
            return;
        }

        let nickname = self.with_player(entity_id, |p| p.nickname.clone());
        let nickname = match nickname {
            Some(n) => n,
            None => return,
        };

        // Check for commands
        if message.starts_with('/') {
            self.handle_command(entity_id, &nickname, &message[1..]);
            return;
        }

        // Broadcast chat
        let chat = ServerPacket::ChatMessage(S2CChatMessage {
            message: format!("<{}> {}", nickname, message),
            sender: Some(nickname),
            chat_type: ChatType::Player,
        });
        self.broadcast_packet(&chat, None);
    }

    pub fn handle_keep_alive_response(&self, entity_id: EntityId, id: u64) {
        self.with_player_mut(entity_id, |p| {
            if p.last_keep_alive_id == id {
                let elapsed = p.last_keep_alive_time.elapsed();
                p.ping_ms = elapsed.as_millis().min(9999) as u16;
            }
        });
    }

    pub fn update_held_item(&self, entity_id: EntityId, slot: u8) {
        if slot > 8 { return; }
        self.with_player_mut(entity_id, |p| {
            p.held_slot = slot;
        });
    }

    pub fn handle_entity_interact(&self, entity_id: EntityId, interact: C2SEntityInteract) {
        match interact.action {
            EntityInteractAction::Attack => {
                self.handle_entity_attack(entity_id, interact.entity_id);
            }
            EntityInteractAction::Interact => {
                // Right-click on entity (e.g. trade with villager, ride horse)
                // Stub for v0.0.1
            }
        }
    }

    pub fn handle_use_item(&self, _entity_id: EntityId, _use_item: C2SUseItem) {
        // Stub: eat food, throw projectile, etc.
    }

    pub fn handle_click_slot(&self, _entity_id: EntityId, _click: C2SClickSlot) {
        // Stub: inventory click handling
    }

    pub fn handle_close_window(&self, _entity_id: EntityId, _window_id: u8) {
        // Stub: close container
    }

    pub fn update_client_settings(&self, entity_id: EntityId, settings: C2SClientSettings) {
        self.with_player_mut(entity_id, |p| {
            p.view_distance = settings.view_distance.min(skycraft_protocol::MAX_VIEW_DISTANCE);
        });
    }

    pub fn handle_player_action(&self, entity_id: EntityId, action: C2SPlayerAction) {
        match action.action {
            PlayerAction::StartDigging => {
                self.handle_block_dig(entity_id, C2SBlockDig {
                    action: DiggingAction::Start,
                    position: action.position,
                    face: action.face,
                });
            }
            PlayerAction::DropItem | PlayerAction::DropItemStack => {
                // Stub: drop item from inventory
            }
            PlayerAction::SwapHands => {
                // Swap main hand and offhand
                self.with_player_mut(entity_id, |p| {
                    let main_slot = p.held_slot as usize;
                    let off_slot = crate::player::OFFHAND_SLOT;
                    p.inventory.swap(main_slot, off_slot);
                });
            }
            _ => {}
        }
    }

    // Sky Craft specific handlers
    pub fn handle_place_marker(&self, _entity_id: EntityId, _marker: C2SPlaceMarker) {
        // Stub: store marker for player
    }

    pub fn handle_remove_marker(&self, _entity_id: EntityId, _marker: C2SRemoveMarker) {
        // Stub: remove marker
    }

    pub fn handle_grappling_hook(&self, entity_id: EntityId, hook: C2SUseGrapplingHook) {
        // Check player has grappling hook in hand and placement lock not active
        let can_use = self.with_player(entity_id, |p| {
            !p.has_debuff(&MobDebuff::PlacementLock(0))
        }).unwrap_or(false);

        if !can_use { return; }

        // Check target block is solid
        if self.world.get_block(hook.target) == 0 {
            return;
        }

        // Calculate pull velocity toward target
        let player_pos = self.with_player(entity_id, |p| p.position);
        if let Some(pos) = player_pos {
            let target = EntityPos::new(
                hook.target.x as f64 + 0.5,
                hook.target.y as f64 + 0.5,
                hook.target.z as f64 + 0.5,
            );
            let dx = target.x - pos.x;
            let dy = target.y - pos.y;
            let dz = target.z - pos.z;
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();

            if dist > 0.5 && dist <= 20.0 {
                let speed = 0.4; // blocks/tick = 8 blocks/sec
                let vel = Velocity {
                    x: dx / dist * speed,
                    y: dy / dist * speed,
                    z: dz / dist * speed,
                };
                self.with_player_mut(entity_id, |p| {
                    p.velocity = vel;
                });
            }
        }
    }

    pub fn handle_emergency_recall(&self, entity_id: EntityId) {
        // Teleport player to bed or spawn, drop all items at current location
        let (pos, respawn) = match self.with_player(entity_id, |p| {
            (p.position, p.respawn_position())
        }) {
            Some(v) => v,
            None => return,
        };

        // Drop all inventory items at current location
        self.with_player_mut(entity_id, |p| {
            for slot in p.inventory.iter_mut() {
                *slot = None; // simplified: items just disappear for now
            }
            p.position = respawn;
        });

        // Teleport
        let tp = ServerPacket::PlayerPositionAndLook(S2CPlayerPositionAndLook {
            x: respawn.x, y: respawn.y, z: respawn.z,
            yaw: 0.0, pitch: 0.0,
        });
        self.send_to_player(entity_id, tp);
    }

    // ── Internal Logic ──────────────────────────────────────────────────────

    fn break_block(&self, entity_id: EntityId, pos: BlockPos) {
        let old_block = self.world.set_block(pos, 0); // set to air
        if old_block == 0 { return; } // was already air

        // Broadcast block change
        let change = ServerPacket::BlockChange(S2CBlockChange {
            position: pos,
            block_state: 0,
        });
        self.broadcast_packet(&change, None);

        // Update player stats
        self.with_player_mut(entity_id, |p| {
            p.statistics.blocks_broken += 1;
            p.add_exhaustion(0.005);
            p.digging_block = None;
            p.digging_progress = 0.0;
        });

        // TODO: drop items based on block type and tool
        // TODO: check gravity blocks (sand, gravel) above
    }

    fn apply_void_damage(&self, entity_id: EntityId, pos: EntityPos) {
        if pos.y < physics::VOID_KILL_Y {
            // Instant death
            self.kill_player(entity_id, DeathCause::VoidFall);
            return;
        }

        // 4 HP/sec = 0.2 HP/tick
        let died = self.with_player_mut(entity_id, |p| {
            p.take_damage(0.2, DeathCause::VoidFall)
        }).unwrap_or(false);

        if died {
            self.kill_player(entity_id, DeathCause::VoidFall);
        } else {
            // Send health update
            let health = self.with_player(entity_id, |p| {
                (p.health, p.food, p.saturation)
            });
            if let Some((hp, food, sat)) = health {
                self.send_to_player(entity_id, ServerPacket::UpdateHealth(S2CUpdateHealth {
                    health: hp, food, saturation: sat,
                }));
            }
        }
    }

    fn kill_player(&self, entity_id: EntityId, cause: DeathCause) {
        let (nickname, pos) = match self.with_player_mut(entity_id, |p| {
            p.is_dead = true;
            p.health = 0.0;
            p.statistics.deaths += 1;
            if matches!(cause, DeathCause::VoidFall) {
                p.statistics.void_deaths += 1;
            }
            if matches!(cause, DeathCause::WindBlown) {
                p.statistics.wind_deaths += 1;
            }
            (p.nickname.clone(), p.position)
        }) {
            Some(v) => v,
            None => return,
        };

        // Send death info to player
        self.send_to_player(entity_id, ServerPacket::DeathInfo(S2CDeathInfo {
            cause: cause.clone(),
            death_position: pos,
            score: 0,
        }));

        // Broadcast death message
        let msg = match &cause {
            DeathCause::VoidFall => format!("{} fell into the void", nickname),
            DeathCause::WindBlown => format!("{} was blown off a bridge by wind", nickname),
            DeathCause::VoidLightning => format!("{} was struck by void lightning", nickname),
            DeathCause::EntityKill { entity_name, ring } => {
                format!("{} was killed by {} [Ring {}]", nickname, entity_name, ring)
            }
            DeathCause::PlayerKill { killer } => format!("{} was killed by {}", nickname, killer),
            DeathCause::FallDamage => format!("{} hit the ground too hard", nickname),
            DeathCause::Drowning => format!("{} drowned", nickname),
            DeathCause::Fire => format!("{} burned to death", nickname),
            DeathCause::Starvation => format!("{} starved to death", nickname),
            DeathCause::Explosion => format!("{} blew up", nickname),
            DeathCause::Other { message } => format!("{} {}", nickname, message),
        };

        let chat = ServerPacket::ChatMessage(S2CChatMessage {
            message: msg,
            sender: None,
            chat_type: ChatType::System,
        });
        self.broadcast_packet(&chat, None);

        // TODO: drop inventory items at death position
        // TODO: handle respawn when client sends respawn request
    }

    fn handle_entity_attack(&self, attacker_id: EntityId, target_id: EntityId) {
        // Check if target is a mob
        if let Some(entry) = self.entities.get(&target_id) {
            if let Ok(mut entity) = entry.value().write() {
                // Calculate damage from attacker's held weapon
                let damage = self.with_player(attacker_id, |p| {
                    // Simplified damage calculation
                    let base = 1.0f32; // hand damage
                    // TODO: look up weapon damage from held item
                    base
                }).unwrap_or(1.0);

                let died = entity.take_damage(damage);

                // Send damage animation
                let anim = ServerPacket::EntityAnimation(S2CEntityAnimation {
                    entity_id: target_id,
                    animation: AnimationType::TakeDamage,
                });
                self.broadcast_packet(&anim, None);

                if died {
                    // Remove entity
                    let destroy = ServerPacket::DestroyEntities(S2CDestroyEntities {
                        entity_ids: vec![target_id],
                    });
                    self.broadcast_packet(&destroy, None);

                    // Update kill stats
                    self.with_player_mut(attacker_id, |p| {
                        p.statistics.mobs_killed += 1;
                    });

                    // TODO: drop loot
                    // TODO: award XP
                }
            }
        }

        // Check if target is another player (PvP)
        if self.config.pvp {
            if self.players.contains_key(&target_id) {
                // TODO: PvP damage
            }
        }
    }

    fn handle_command(&self, entity_id: EntityId, nickname: &str, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return; }

        match parts[0] {
            "help" => {
                let msg = "Available commands: /help, /list, /spawn, /seed, /ring, /stats";
                self.send_system_message(entity_id, msg);
            }
            "list" => {
                let names: Vec<String> = self.nickname_map.iter()
                    .map(|e| e.key().clone()).collect();
                let msg = format!("Online ({}/{}): {}", names.len(), self.config.max_players, names.join(", "));
                self.send_system_message(entity_id, &msg);
            }
            "seed" => {
                let msg = format!("World seed: {}", self.config.seed);
                self.send_system_message(entity_id, &msg);
            }
            "ring" => {
                let ring = self.with_player(entity_id, |p| p.current_ring).unwrap_or(0);
                let msg = format!("You are in Ring {}", ring);
                self.send_system_message(entity_id, &msg);
            }
            "stats" => {
                if let Some(msg) = self.with_player(entity_id, |p| {
                    format!(
                        "Stats: Ring {} | Islands {} | Mobs {} | Deaths {} | Distance {:.1}km",
                        p.statistics.highest_ring,
                        p.statistics.islands_explored,
                        p.statistics.mobs_killed,
                        p.statistics.deaths,
                        p.statistics.distance_walked / 1000.0,
                    )
                }) {
                    self.send_system_message(entity_id, &msg);
                }
            }
            "spawn" => {
                let spawn = self.world.get_spawn_position();
                self.with_player_mut(entity_id, |p| {
                    p.position = spawn;
                });
                self.send_to_player(entity_id, ServerPacket::PlayerPositionAndLook(S2CPlayerPositionAndLook {
                    x: spawn.x, y: spawn.y, z: spawn.z,
                    yaw: 0.0, pitch: 0.0,
                }));
                self.send_system_message(entity_id, "Teleported to spawn");
            }
            _ => {
                self.send_system_message(entity_id, &format!("Unknown command: /{}", parts[0]));
            }
        }
    }

    fn send_system_message(&self, entity_id: EntityId, msg: &str) {
        self.send_to_player(entity_id, ServerPacket::ChatMessage(S2CChatMessage {
            message: msg.to_string(),
            sender: None,
            chat_type: ChatType::System,
        }));
    }

    /// Spawn cows near a position if none exist in the area. Called when a player joins.
    pub fn spawn_cows_near(&self, center: EntityPos) {
        // Cow entity type ID from entities.json: cow = 11
        const COW_ENTITY_TYPE: EntityTypeId = 11;
        const NUM_COWS: usize = 5;
        const SPAWN_RADIUS: f64 = 16.0;

        for i in 0..NUM_COWS {
            let angle = (i as f64) * std::f64::consts::TAU / NUM_COWS as f64;
            let r = 6.0 + (i as f64) * 2.0;
            let cx = center.x + angle.cos() * r;
            let cz = center.z + angle.sin() * r;

            // Find ground level
            let ground_y = self.find_ground_y(cx, cz);
            if ground_y < 0 { continue; }

            let pos = EntityPos::new(cx, ground_y as f64 + 1.0, cz);
            let id = self.next_entity_id.fetch_add(1, Ordering::Relaxed);
            let mut entity = Entity::new_mob(id, COW_ENTITY_TYPE, pos, 10.0, 3.0, 1.0);

            // Set as passive mob
            if let EntityKind::Mob(ref mut mob) = entity.kind {
                mob.mob_category = MobCategory::Passive;
                mob.ai.wander_cooldown = (id % 40) as u32; // stagger wandering
            }

            let spawn_pkt = ServerPacket::SpawnEntity(S2CSpawnEntity {
                entity_id: id,
                entity_type: COW_ENTITY_TYPE,
                position: pos,
                rotation: Rotation { yaw: (i as f32) * 72.0, pitch: 0.0 },
                velocity: Velocity { x: 0.0, y: 0.0, z: 0.0 },
            });
            self.broadcast_packet(&spawn_pkt, None);
            self.entities.insert(id, RwLock::new(entity));
        }
    }

    /// Find solid ground Y at given XZ. Returns -1 if not found.
    fn find_ground_y(&self, x: f64, z: f64) -> i32 {
        let bx = x.floor() as i32;
        let bz = z.floor() as i32;
        for y in (50..80).rev() {
            let block = self.world.get_block(BlockPos::new(bx, y, bz));
            let above = self.world.get_block(BlockPos::new(bx, y + 1, bz));
            if block != 0 && above == 0 {
                return y;
            }
        }
        -1
    }

    /// Send all existing entities to a newly joined player.
    pub fn send_entities_to_player(&self, entity_id: EntityId) {
        for entry in self.entities.iter() {
            if let Ok(entity) = entry.value().read() {
                if entity.is_dead { continue; }
                let pkt = ServerPacket::SpawnEntity(S2CSpawnEntity {
                    entity_id: entity.id,
                    entity_type: entity.entity_type,
                    position: entity.position,
                    rotation: entity.rotation,
                    velocity: entity.velocity,
                });
                self.send_to_player(entity_id, pkt);
            }
        }
    }
}

// ─── Game Loop ──────────────────────────────────────────────────────────────

/// Main game loop. Runs at 20 TPS.
pub async fn game_loop(state: Arc<GameState>) {
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(MS_PER_TICK as u64));

    loop {
        interval.tick().await;
        tick(&state);
    }
}

/// Single game tick. Called 20 times per second.
fn tick(state: &GameState) {
    let tick = state.world_age.fetch_add(1, Ordering::Relaxed);

    // Update time of day (0-24000 cycle)
    if state.config.do_daylight_cycle {
        let tod = state.time_of_day.fetch_add(1, Ordering::Relaxed);
        if tod >= 24000 {
            state.time_of_day.store(0, Ordering::Relaxed);
        }
    }

    // Send time update every 20 ticks (1 sec)
    if tick % 20 == 0 {
        let time_packet = ServerPacket::TimeUpdate(S2CTimeUpdate {
            world_age: tick + 1,
            time_of_day: state.time_of_day.load(Ordering::Relaxed),
        });
        state.broadcast_packet(&time_packet, None);
    }

    // Update weather
    tick_weather(state, tick);

    // Update wind
    tick_wind(state, tick);

    // Tick all players
    tick_players(state, tick);

    // Tick all entities
    tick_entities(state, tick);

    // Keep-alive every 15 seconds (300 ticks)
    if tick % 300 == 0 {
        tick_keep_alive(state);
    }
}

fn tick_weather(state: &GameState, _tick: u64) {
    let remaining = state.weather_ticks_remaining.fetch_sub(1, Ordering::Relaxed);
    if remaining <= 1 {
        // Change weather
        let current = state.current_weather();
        let new_weather = match current {
            Weather::Clear => {
                // 20% chance of rain, 5% thunder
                let r: u32 = rand::random::<u32>() % 100;
                if r < 5 { Weather::Thunder } else if r < 25 { Weather::Rain } else { Weather::Clear }
            }
            Weather::Rain => {
                let r: u32 = rand::random::<u32>() % 100;
                if r < 10 { Weather::Thunder } else if r < 60 { Weather::Clear } else { Weather::Rain }
            }
            Weather::Thunder => {
                let r: u32 = rand::random::<u32>() % 100;
                if r < 50 { Weather::Rain } else { Weather::Clear }
            }
        };

        *state.weather.write().unwrap() = new_weather;
        let duration = 6000 + rand::random::<u32>() % 12000; // 5-15 min
        state.weather_ticks_remaining.store(duration, Ordering::Relaxed);

        let packet = ServerPacket::WeatherChange(S2CWeatherChange { weather: new_weather });
        state.broadcast_packet(&packet, None);
    }
}

fn tick_wind(state: &GameState, tick: u64) {
    // Recalculate wind every 20 ticks
    if tick % 20 != 0 { return; }

    // Find highest ring among all players for wind calculation
    let max_ring = state.players.iter()
        .filter_map(|e| e.value().read().ok().map(|p| p.current_ring))
        .max()
        .unwrap_or(0);

    let wind = physics::calculate_wind(max_ring, tick);
    *state.wind.write().unwrap() = wind;

    // Send wind update to players who need it (ring > 0)
    for entry in state.players.iter() {
        if let Ok(player) = entry.value().read() {
            if player.current_ring > 0 {
                player.send_packet(ServerPacket::WindUpdate(S2CWindUpdate { wind }));
            }
        }
    }
}

fn tick_players(state: &GameState, tick: u64) {
    let player_ids: Vec<EntityId> = state.players.iter().map(|e| *e.key()).collect();

    for &eid in &player_ids {
        state.with_player_mut(eid, |player| {
            if player.is_dead { return; }

            player.statistics.play_time_ticks += 1;

            // Tick invulnerability
            if player.invulnerability_ticks > 0 {
                player.invulnerability_ticks -= 1;
            }
            player.ticks_since_last_attack += 1;
            player.ticks_since_last_damage += 1;

            // Tick debuffs
            player.active_debuffs.retain_mut(|d| {
                if d.remaining_ticks > 0 {
                    d.remaining_ticks -= 1;
                    true
                } else {
                    false
                }
            });

            // Tick potion effects
            player.active_effects.retain_mut(|e| {
                if e.duration > 0 {
                    e.duration -= 1;
                    true
                } else {
                    false
                }
            });

            // Hunger tick
            player.food_tick_timer += 1;

            // Natural regeneration
            if player.health < player.max_health && player.food >= 18 {
                player.heal_tick_timer += 1;
                if player.food == 20 && player.saturation > 0.0 {
                    // Rapid regen: 1 HP every 10 ticks
                    if player.heal_tick_timer >= 10 {
                        player.health = (player.health + 1.0).min(player.max_health);
                        player.add_exhaustion(6.0);
                        player.heal_tick_timer = 0;
                    }
                } else if player.heal_tick_timer >= 80 {
                    // Slow regen: 1 HP every 80 ticks (4 sec)
                    player.health = (player.health + 1.0).min(player.max_health);
                    player.add_exhaustion(6.0);
                    player.heal_tick_timer = 0;
                }
            }

            // Starvation damage
            if player.food == 0 && player.food_tick_timer >= 80 {
                player.food_tick_timer = 0;
                let min_health = match player.difficulty {
                    Difficulty::Easy => 10.0,
                    Difficulty::Normal => 1.0,
                    Difficulty::Hard => 0.0,
                    Difficulty::Peaceful => 20.0,
                };
                if player.health > min_health {
                    player.health -= 1.0;
                }
            }

            // Phantom spawn timer
            player.ticks_without_sleep += 1;
        });

        // Send health updates periodically (every second)
        if tick % 20 == 0 {
            if let Some((hp, food, sat)) = state.with_player(eid, |p| (p.health, p.food, p.saturation)) {
                state.send_to_player(eid, ServerPacket::UpdateHealth(S2CUpdateHealth {
                    health: hp, food, saturation: sat,
                }));
            }
        }

        // Send ring updates periodically
        if tick % 40 == 0 {
            if let Some(ring) = state.with_player(eid, |p| p.current_ring) {
                state.send_to_player(eid, ServerPacket::RingUpdate(S2CRingUpdate {
                    ring,
                    island: None, // TODO: detect current island
                }));
            }
        }
    }
}

fn tick_entities(state: &GameState, tick: u64) {
    let entity_ids: Vec<EntityId> = state.entities.iter().map(|e| *e.key()).collect();

    for &eid in &entity_ids {
        // Snapshot old position for delta movement packet
        let old_pos = state.entities.get(&eid)
            .and_then(|e| e.value().read().ok().map(|en| en.position));

        if let Some(entry) = state.entities.get(&eid) {
            if let Ok(mut entity) = entry.value().write() {
                if entity.is_dead { continue; }

                // Tick invulnerability
                if entity.no_damage_ticks > 0 {
                    entity.no_damage_ticks -= 1;
                }

                // Gravity for mobs: keep them on ground
                let is_mob = matches!(entity.kind, EntityKind::Mob(_));
                if is_mob {
                    // Check block below
                    let below = BlockPos::new(
                        entity.position.x.floor() as i32,
                        (entity.position.y - 0.05) as i32,
                        entity.position.z.floor() as i32,
                    );
                    let on_ground = state.world.get_block(below) != 0;
                    entity.on_ground = on_ground;
                    if !on_ground {
                        entity.velocity.y -= 0.08; // gravity
                        entity.velocity.y *= 0.98;
                        entity.position.y += entity.velocity.y;
                    } else {
                        entity.velocity.y = 0.0;
                        // Snap to ground
                        let ground_y = below.y as f64 + 1.0;
                        if entity.position.y < ground_y {
                            entity.position.y = ground_y;
                        }
                    }
                } else if !entity.on_ground {
                    // Non-mob entities: full gravity
                    entity.velocity = physics::apply_gravity(entity.velocity, false);
                    entity.position.x += entity.velocity.x;
                    entity.position.y += entity.velocity.y;
                    entity.position.z += entity.velocity.z;
                }

                // Void kill for entities
                if entity.position.y < physics::VOID_KILL_Y {
                    entity.is_dead = true;
                }

                // Fire tick
                if entity.fire_ticks > 0 {
                    entity.fire_ticks -= 1;
                    if entity.fire_ticks == 0 {
                        entity.is_on_fire = false;
                    }
                }

                // Despawn timer for items
                if let EntityKind::Item(ref mut item_data) = entity.kind {
                    item_data.age += 1;
                    if item_data.pickup_delay > 0 {
                        item_data.pickup_delay -= 1;
                    }
                }
                if matches!(entity.kind, EntityKind::Item(ref d) if d.age >= 6000) {
                    entity.is_dead = true;
                }

                // Basic mob AI tick
                let is_passive_mob = matches!(&entity.kind, EntityKind::Mob(m) if m.mob_category == MobCategory::Passive);
                {
                    if let EntityKind::Mob(ref mut mob) = entity.kind {
                        mob.ticks_since_attack += 1;
                        mob.despawn_timer += 1;
                        if mob.despawn_timer > 12000 {
                            entity.is_dead = true;
                        }
                    }
                }

                // Passive mob wandering AI (separate borrow scope)
                if is_passive_mob && !entity.is_dead {
                    // Snapshot values needed for computation before borrowing mob
                    let eid_f = entity.id as f64;
                    let ex = entity.position.x;
                    let ey = entity.position.y;
                    let ez = entity.position.z;
                    let eid_u = entity.id;

                    if let EntityKind::Mob(ref mut mob) = entity.kind {
                        if mob.ai.wander_cooldown > 0 {
                            mob.ai.wander_cooldown -= 1;
                        } else {
                            match mob.ai.state {
                                AIState::Idle => {
                                    let angle = (eid_f * 2.399963 + ex * 0.1) % std::f64::consts::TAU;
                                    let dist = 3.0 + (eid_f * 1.618) % 5.0;
                                    mob.ai.wander_target = Some(BlockPos::new(
                                        (ex + angle.cos() * dist) as i32,
                                        ey as i32,
                                        (ez + angle.sin() * dist) as i32,
                                    ));
                                    mob.ai.state = AIState::Wandering;
                                    mob.ai.wander_cooldown = 20 + (eid_u % 20) as u32;
                                }
                                AIState::Wandering => {
                                    if let Some(target) = mob.ai.wander_target {
                                        let tx = target.x as f64 + 0.5;
                                        let tz = target.z as f64 + 0.5;
                                        let dx = tx - ex;
                                        let dz = tz - ez;
                                        let dist = (dx * dx + dz * dz).sqrt();
                                        if dist < 0.5 {
                                            mob.ai.state = AIState::Idle;
                                            mob.ai.wander_cooldown = 40 + (eid_u % 60) as u32;
                                            entity.velocity.x = 0.0;
                                            entity.velocity.z = 0.0;
                                        } else {
                                            let speed = 0.08;
                                            entity.velocity.x = dx / dist * speed;
                                            entity.velocity.z = dz / dist * speed;
                                            entity.rotation.yaw = (dz.atan2(dx) as f32).to_degrees();
                                        }
                                    } else {
                                        mob.ai.state = AIState::Idle;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    // Move horizontally with AABB collision (same approach as player)
                    // Cow hitbox: 0.45 half-width, 1.4 tall
                    const COW_HW: f64 = 0.45;
                    const COW_H: f64 = 1.4;

                    let cow_blocked = |cx: f64, cy: f64, cz: f64| -> bool {
                        let x0 = (cx - COW_HW).floor() as i32;
                        let x1 = (cx + COW_HW).floor() as i32;
                        let y0 = cy.floor() as i32;
                        let y1 = (cy + COW_H - 0.01).floor() as i32;
                        let z0 = (cz - COW_HW).floor() as i32;
                        let z1 = (cz + COW_HW).floor() as i32;
                        for bx in x0..=x1 {
                            for by in y0..=y1 {
                                for bz in z0..=z1 {
                                    if state.world.get_block(BlockPos::new(bx, by, bz)) != 0 {
                                        return true;
                                    }
                                }
                            }
                        }
                        false
                    };

                    let cur_x = entity.position.x;
                    let cur_y = entity.position.y;
                    let cur_z = entity.position.z;
                    let new_x = cur_x + entity.velocity.x;
                    let new_z = cur_z + entity.velocity.z;

                    let can_x = !cow_blocked(new_x, cur_y, cur_z);
                    let can_z = !cow_blocked(if can_x { new_x } else { cur_x }, cur_y, new_z);

                    if can_x {
                        entity.position.x = new_x;
                    } else {
                        entity.velocity.x = 0.0;
                        // Stuck — reset to idle so AI picks a new target
                        if let EntityKind::Mob(ref mut mob) = entity.kind {
                            mob.ai.state = AIState::Idle;
                            mob.ai.wander_cooldown = 10;
                        }
                    }
                    if can_z {
                        entity.position.z = new_z;
                    } else {
                        entity.velocity.z = 0.0;
                        if let EntityKind::Mob(ref mut mob) = entity.kind {
                            mob.ai.state = AIState::Idle;
                            mob.ai.wander_cooldown = 10;
                        }
                    }
                }
            }
        }

        // Broadcast movement every 2 ticks for mobs (10 Hz)
        if tick % 2 == 0 {
            if let (Some(old), Some(entry)) = (old_pos, state.entities.get(&eid)) {
                if let Ok(entity) = entry.value().read() {
                    if !entity.is_dead {
                        let dx = ((entity.position.x - old.x) * 4096.0) as i16;
                        let dy = ((entity.position.y - old.y) * 4096.0) as i16;
                        let dz = ((entity.position.z - old.z) * 4096.0) as i16;
                        if dx != 0 || dy != 0 || dz != 0 {
                            let pkt = ServerPacket::EntityMoveAndLook(S2CEntityMoveAndLook {
                                entity_id: eid,
                                dx, dy, dz,
                                yaw: entity.rotation.yaw,
                                pitch: entity.rotation.pitch,
                                on_ground: entity.on_ground,
                            });
                            state.broadcast_packet(&pkt, None);
                        }
                    }
                }
            }
        }

        // Clean up dead entities
        if let Some(entry) = state.entities.get(&eid) {
            let is_dead = entry.value().read().map(|e| e.is_dead).unwrap_or(true);
            if is_dead {
                state.entities.remove(&eid);
                let destroy = ServerPacket::DestroyEntities(S2CDestroyEntities {
                    entity_ids: vec![eid],
                });
                state.broadcast_packet(&destroy, None);
            }
        }
    }
}

fn tick_keep_alive(state: &GameState) {
    let ka_id = state.keep_alive_counter.fetch_add(1, Ordering::Relaxed);
    let packet = ServerPacket::KeepAlive(S2CKeepAlive { id: ka_id });

    for entry in state.players.iter() {
        if let Ok(mut player) = entry.value().write() {
            player.last_keep_alive_id = ka_id;
            player.last_keep_alive_time = std::time::Instant::now();
            player.send_packet(packet.clone());
        }
    }
}
