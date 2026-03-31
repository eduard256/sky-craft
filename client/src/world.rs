// Client-side world representation. Stores chunks received from server,
// provides block access for rendering. No game logic -- just data.

use std::collections::HashMap;
use skycraft_protocol::types::*;
use skycraft_protocol::packets::*;
use tracing::debug;

/// Client-side world: chunk cache + entity positions + player state.
pub struct ClientWorld {
    /// Loaded chunk sections indexed by chunk position.
    chunks: HashMap<ChunkPos, ChunkSection>,
    /// Known entities (mobs, items, other players).
    entities: HashMap<EntityId, ClientEntity>,
    /// Local player state (from server updates).
    pub player: LocalPlayer,
    /// Current weather.
    pub weather: Weather,
    /// World time.
    pub world_age: u64,
    pub time_of_day: u32,
    /// Sky Craft specific state.
    pub current_ring: RingNumber,
    pub wind: WindState,
    pub island_info: Option<IslandInfo>,
    pub active_debuffs: Vec<MobDebuff>,
}

/// Minimal entity data for client-side rendering.
pub struct ClientEntity {
    pub id: EntityId,
    pub entity_type: EntityTypeId,
    pub position: EntityPos,
    pub rotation: Rotation,
    pub velocity: Velocity,
    pub on_ground: bool,
    pub custom_name: Option<String>,
    pub is_on_fire: bool,
}

/// Local player state received from server.
pub struct LocalPlayer {
    pub position: EntityPos,
    pub rotation: Rotation,
    pub health: f32,
    pub food: u8,
    pub saturation: f32,
    pub xp_bar: f32,
    pub xp_level: u16,
    pub game_mode: GameMode,
    pub inventory: Vec<Slot>,
    pub held_slot: u8,
    pub on_ground: bool,
}

impl ClientWorld {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            entities: HashMap::new(),
            player: LocalPlayer::default(),
            weather: Weather::Clear,
            world_age: 0,
            time_of_day: 0,
            current_ring: 0,
            wind: WindState { direction: 0.0, strength: 0.0, gusting: false },
            island_info: None,
            active_debuffs: Vec::new(),
        }
    }

    /// Get block at world position from loaded chunks.
    pub fn get_block(&self, pos: BlockPos) -> BlockStateId {
        let chunk_pos = pos.to_chunk_pos();
        let (lx, ly, lz) = pos.chunk_local();
        match self.chunks.get(&chunk_pos) {
            Some(section) => section.get_block(lx, ly, lz),
            None => 0, // unloaded = air
        }
    }

    /// Check if a chunk is loaded.
    pub fn is_chunk_loaded(&self, pos: &ChunkPos) -> bool {
        self.chunks.contains_key(pos)
    }

    /// Get a chunk section by position.
    pub fn get_chunk(&self, pos: &ChunkPos) -> Option<ChunkSection> {
        self.chunks.get(pos).cloned()
    }

    /// Iterator over all loaded chunk positions.
    pub fn loaded_chunk_positions(&self) -> impl Iterator<Item = ChunkPos> + '_ {
        self.chunks.keys().copied()
    }

    /// Number of loaded chunks.
    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Process a packet received from the server.
    pub fn handle_server_packet(&mut self, packet: ServerPacket) {
        match packet {
            // ── World ──
            ServerPacket::ChunkData(data) => {
                self.chunks.insert(data.chunk_pos, data.section);
            }
            ServerPacket::UnloadChunk(unload) => {
                self.chunks.remove(&unload.chunk_pos);
            }
            ServerPacket::BlockChange(change) => {
                let chunk_pos = change.position.to_chunk_pos();
                let (lx, ly, lz) = change.position.chunk_local();
                if let Some(section) = self.chunks.get_mut(&chunk_pos) {
                    set_block_in_section(section, lx, ly, lz, change.block_state);
                }
            }
            ServerPacket::MultiBlockChange(multi) => {
                if let Some(section) = self.chunks.get_mut(&multi.chunk_pos) {
                    for (lx, ly, lz, state) in multi.changes {
                        set_block_in_section(section, lx, ly, lz, state);
                    }
                }
            }

            // ── Entities ──
            ServerPacket::SpawnEntity(spawn) => {
                self.entities.insert(spawn.entity_id, ClientEntity {
                    id: spawn.entity_id,
                    entity_type: spawn.entity_type,
                    position: spawn.position,
                    rotation: spawn.rotation,
                    velocity: spawn.velocity,
                    on_ground: false,
                    custom_name: None,
                    is_on_fire: false,
                });
            }
            ServerPacket::SpawnPlayer(spawn) => {
                self.entities.insert(spawn.entity_id, ClientEntity {
                    id: spawn.entity_id,
                    entity_type: 0,
                    position: spawn.position,
                    rotation: spawn.rotation,
                    velocity: Velocity { x: 0.0, y: 0.0, z: 0.0 },
                    on_ground: true,
                    custom_name: Some(spawn.nickname),
                    is_on_fire: false,
                });
            }
            ServerPacket::EntityMove(mv) => {
                if let Some(entity) = self.entities.get_mut(&mv.entity_id) {
                    entity.position.x += mv.dx as f64 / 4096.0;
                    entity.position.y += mv.dy as f64 / 4096.0;
                    entity.position.z += mv.dz as f64 / 4096.0;
                    entity.on_ground = mv.on_ground;
                }
            }
            ServerPacket::EntityLook(look) => {
                if let Some(entity) = self.entities.get_mut(&look.entity_id) {
                    entity.rotation.yaw = look.yaw;
                    entity.rotation.pitch = look.pitch;
                }
            }
            ServerPacket::EntityMoveAndLook(ml) => {
                if let Some(entity) = self.entities.get_mut(&ml.entity_id) {
                    entity.position.x += ml.dx as f64 / 4096.0;
                    entity.position.y += ml.dy as f64 / 4096.0;
                    entity.position.z += ml.dz as f64 / 4096.0;
                    entity.rotation.yaw = ml.yaw;
                    entity.rotation.pitch = ml.pitch;
                    entity.on_ground = ml.on_ground;
                }
            }
            ServerPacket::EntityTeleport(tp) => {
                if let Some(entity) = self.entities.get_mut(&tp.entity_id) {
                    entity.position = tp.position;
                    entity.rotation = tp.rotation;
                    entity.on_ground = tp.on_ground;
                }
            }
            ServerPacket::EntityVelocity(vel) => {
                if let Some(entity) = self.entities.get_mut(&vel.entity_id) {
                    entity.velocity = vel.velocity;
                }
            }
            ServerPacket::DestroyEntities(destroy) => {
                for id in destroy.entity_ids {
                    self.entities.remove(&id);
                }
            }
            ServerPacket::EntityMetadata(meta) => {
                if let Some(entity) = self.entities.get_mut(&meta.entity_id) {
                    if let Some(name) = meta.custom_name {
                        entity.custom_name = Some(name);
                    }
                    entity.is_on_fire = meta.is_on_fire;
                }
            }

            // ── Player state ──
            ServerPacket::UpdateHealth(h) => {
                self.player.health = h.health;
                self.player.food = h.food;
                self.player.saturation = h.saturation;
            }
            ServerPacket::SetExperience(xp) => {
                self.player.xp_bar = xp.bar;
                self.player.xp_level = xp.level;
            }
            ServerPacket::PlayerPositionAndLook(pos) => {
                self.player.position = EntityPos::new(pos.x, pos.y, pos.z);
                self.player.rotation = Rotation { yaw: pos.yaw, pitch: pos.pitch };
            }

            // ── Inventory ──
            ServerPacket::WindowItems(items) => {
                if items.window_id == 0 {
                    self.player.inventory = items.slots;
                }
            }
            ServerPacket::SetSlot(slot) => {
                if slot.window_id == 0 {
                    let idx = slot.slot as usize;
                    if idx < self.player.inventory.len() {
                        self.player.inventory[idx] = slot.item;
                    }
                }
            }

            // ── World state ──
            ServerPacket::TimeUpdate(time) => {
                self.world_age = time.world_age;
                self.time_of_day = time.time_of_day;
            }
            ServerPacket::WeatherChange(w) => {
                self.weather = w.weather;
            }

            // ── Sky Craft ──
            ServerPacket::RingUpdate(ring) => {
                self.current_ring = ring.ring;
                self.island_info = ring.island;
            }
            ServerPacket::WindUpdate(w) => {
                self.wind = w.wind;
            }
            ServerPacket::DebuffApplied(d) => {
                self.active_debuffs.push(d.debuff);
            }
            ServerPacket::DebuffExpired(d) => {
                self.active_debuffs.retain(|debuff| {
                    std::mem::discriminant(debuff) != std::mem::discriminant(&match d.debuff_type {
                        DebuffType::PlacementLock => MobDebuff::PlacementLock(0),
                        DebuffType::MiningLock => MobDebuff::MiningLock(0),
                        DebuffType::InventoryLock => MobDebuff::InventoryLock(0),
                        DebuffType::GravityPull => MobDebuff::GravityPull(0),
                        DebuffType::Fear => MobDebuff::Fear(0),
                        DebuffType::VoidSickness => MobDebuff::VoidSickness(0),
                        DebuffType::SoulDrain => MobDebuff::SoulDrain(0),
                        DebuffType::AnchorBreak => MobDebuff::AnchorBreak,
                    })
                });
            }

            // ── Misc (log but don't process yet) ──
            _ => {
                debug!("Unhandled server packet: {:?}", std::mem::discriminant(&packet));
            }
        }
    }
}

impl Default for LocalPlayer {
    fn default() -> Self {
        Self {
            position: EntityPos::new(0.0, 64.0, 0.0),
            rotation: Rotation { yaw: 0.0, pitch: 0.0 },
            health: 20.0,
            food: 20,
            saturation: 5.0,
            xp_bar: 0.0,
            xp_level: 0,
            game_mode: GameMode::Survival,
            inventory: vec![None; 46],
            held_slot: 0,
            on_ground: true,
        }
    }
}

/// Set a block in a chunk section, expanding palette if needed.
fn set_block_in_section(section: &mut ChunkSection, lx: u8, ly: u8, lz: u8, state: BlockStateId) {
    let index = (ly as usize) * 256 + (lz as usize) * 16 + (lx as usize);

    if section.blocks.is_empty() {
        let current = section.palette[0];
        if current == state { return; }
        section.blocks = vec![0; ChunkSection::VOLUME];
    }

    if let Some(palette_idx) = section.palette.iter().position(|&s| s == state) {
        section.blocks[index] = palette_idx as u16;
    } else {
        let new_idx = section.palette.len() as u16;
        section.palette.push(state);
        section.blocks[index] = new_idx;
    }
}
