// Player state. Each connected player has one Player struct on the server.

use skycraft_protocol::types::*;
use skycraft_protocol::packets::ServerPacket;
use std::collections::VecDeque;
use std::sync::Mutex;

/// Maximum inventory slots (36 main + 4 armor + 1 offhand + 4 crafting + 1 output = 46).
pub const INVENTORY_SIZE: usize = 46;

/// Hotbar slot range in the inventory array.
pub const HOTBAR_START: usize = 0;
pub const HOTBAR_END: usize = 9;

/// Armor slot indices.
pub const ARMOR_HELMET: usize = 36;
pub const ARMOR_CHESTPLATE: usize = 37;
pub const ARMOR_LEGGINGS: usize = 38;
pub const ARMOR_BOOTS: usize = 39;

/// Offhand slot.
pub const OFFHAND_SLOT: usize = 40;

pub struct Player {
    // ── Identity ──
    pub uuid: PlayerId,
    pub nickname: String,
    pub entity_id: EntityId,

    // ── Position ──
    pub position: EntityPos,
    pub rotation: Rotation,
    pub on_ground: bool,
    pub velocity: Velocity,

    // ── Stats ──
    pub health: f32,
    pub max_health: f32,
    pub food: u8,
    pub saturation: f32,
    pub exhaustion: f32,
    pub xp_level: u16,
    pub xp_total: u32,
    pub xp_bar: f32,

    // ── State ──
    pub game_mode: GameMode,
    pub difficulty: Difficulty,
    pub is_sneaking: bool,
    pub is_sprinting: bool,
    pub is_flying: bool,
    pub is_dead: bool,

    // ── Inventory ──
    pub inventory: Vec<Slot>,
    pub held_slot: u8,
    pub cursor_item: Slot,

    // ── Spawn ──
    pub spawn_position: EntityPos,
    pub bed_position: Option<BlockPos>,
    pub has_slept: bool,

    // ── Sky Craft ──
    pub current_ring: RingNumber,
    pub active_debuffs: Vec<ActiveDebuff>,
    pub active_effects: Vec<PotionEffect>,
    pub statistics: PlayerStats,

    // ── Network ──
    pub view_distance: u8,
    pub last_keep_alive_id: u64,
    pub last_keep_alive_time: std::time::Instant,
    pub ping_ms: u16,
    outbound_queue: Mutex<VecDeque<ServerPacket>>,

    // ── Timing ──
    pub ticks_since_last_attack: u32,
    pub ticks_since_last_damage: u32,
    pub invulnerability_ticks: u32,
    pub ticks_without_sleep: u64,
    pub food_tick_timer: u32,
    pub heal_tick_timer: u32,

    // ── Digging ──
    pub digging_block: Option<BlockPos>,
    pub digging_progress: f32,
}

/// Active debuff with remaining duration.
pub struct ActiveDebuff {
    pub debuff: MobDebuff,
    pub remaining_ticks: u32,
}

/// Player statistics tracked by server.
pub struct PlayerStats {
    pub highest_ring: RingNumber,
    pub islands_explored: u32,
    pub blocks_placed: u64,
    pub blocks_broken: u64,
    pub mobs_killed: u32,
    pub deaths: u32,
    pub void_deaths: u32,
    pub wind_deaths: u32,
    pub distance_walked: f64,
    pub play_time_ticks: u64,
}

impl Player {
    pub fn new(
        uuid: PlayerId,
        nickname: String,
        spawn_pos: EntityPos,
        difficulty: Difficulty,
    ) -> Self {
        Self {
            uuid,
            nickname,
            entity_id: 0, // set by game state when added
            position: spawn_pos,
            rotation: Rotation { yaw: 0.0, pitch: 0.0 },
            on_ground: true,
            velocity: Velocity { x: 0.0, y: 0.0, z: 0.0 },

            health: 20.0,
            max_health: 20.0,
            food: 20,
            saturation: 5.0,
            exhaustion: 0.0,
            xp_level: 0,
            xp_total: 0,
            xp_bar: 0.0,

            game_mode: GameMode::Survival,
            difficulty,
            is_sneaking: false,
            is_sprinting: false,
            is_flying: false,
            is_dead: false,

            inventory: vec![None; INVENTORY_SIZE],
            held_slot: 0,
            cursor_item: None,

            spawn_position: spawn_pos,
            bed_position: None,
            has_slept: false,

            current_ring: 0,
            active_debuffs: Vec::new(),
            active_effects: Vec::new(),
            statistics: PlayerStats::default(),

            view_distance: skycraft_protocol::DEFAULT_VIEW_DISTANCE,
            last_keep_alive_id: 0,
            last_keep_alive_time: std::time::Instant::now(),
            ping_ms: 0,
            outbound_queue: Mutex::new(VecDeque::new()),

            ticks_since_last_attack: 100,
            ticks_since_last_damage: 100,
            invulnerability_ticks: 0,
            ticks_without_sleep: 0,
            food_tick_timer: 0,
            heal_tick_timer: 0,

            digging_block: None,
            digging_progress: 0.0,
        }
    }

    /// Queue a packet to be sent to this player.
    pub fn send_packet(&self, packet: ServerPacket) {
        if let Ok(mut queue) = self.outbound_queue.lock() {
            queue.push_back(packet);
        }
    }

    /// Pop next outbound packet.
    pub fn pop_packet(&self) -> Option<ServerPacket> {
        if let Ok(mut queue) = self.outbound_queue.lock() {
            queue.pop_front()
        } else {
            None
        }
    }

    /// Get the item in the currently held hotbar slot.
    pub fn held_item(&self) -> &Slot {
        &self.inventory[self.held_slot as usize]
    }

    /// Calculate the ring number from current XZ position.
    pub fn calculate_ring(&self) -> RingNumber {
        let dist = self.position.horizontal_distance_to_origin();
        (dist / 500.0) as RingNumber
    }

    /// Check if player has a specific debuff active.
    pub fn has_debuff(&self, check: &MobDebuff) -> bool {
        self.active_debuffs.iter().any(|d| {
            std::mem::discriminant(&d.debuff) == std::mem::discriminant(check)
        })
    }

    /// Respawn position (bed or world spawn).
    pub fn respawn_position(&self) -> EntityPos {
        // If bed exists, use bed position (convert block to entity pos)
        if let Some(bed_pos) = &self.bed_position {
            EntityPos::new(bed_pos.x as f64 + 0.5, bed_pos.y as f64 + 1.0, bed_pos.z as f64 + 0.5)
        } else {
            self.spawn_position
        }
    }

    /// Apply damage to player. Returns true if player died.
    pub fn take_damage(&mut self, amount: f32, source: DeathCause) -> bool {
        if self.invulnerability_ticks > 0 || self.game_mode == GameMode::Creative {
            return false;
        }

        // Apply armor reduction (simplified)
        let armor_points = self.calculate_armor_points();
        let reduced = amount * (1.0 - (armor_points as f32 / 25.0).min(0.8));

        self.health = (self.health - reduced).max(0.0);
        self.invulnerability_ticks = 10; // 0.5 sec
        self.ticks_since_last_damage = 0;

        if self.health <= 0.0 {
            self.is_dead = true;
            true
        } else {
            false
        }
    }

    /// Calculate total armor points from equipped armor.
    fn calculate_armor_points(&self) -> u32 {
        // Simplified: count equipped armor slots, give approximate points
        let mut points = 0u32;
        for slot in [ARMOR_HELMET, ARMOR_CHESTPLATE, ARMOR_LEGGINGS, ARMOR_BOOTS] {
            if self.inventory[slot].is_some() {
                points += 4; // rough average per piece
            }
        }
        points.min(20)
    }

    /// Add exhaustion (hunger system).
    pub fn add_exhaustion(&mut self, amount: f32) {
        self.exhaustion += amount;
        while self.exhaustion >= 4.0 {
            self.exhaustion -= 4.0;
            if self.saturation > 0.0 {
                self.saturation = (self.saturation - 1.0).max(0.0);
            } else if self.food > 0 {
                self.food -= 1;
            }
        }
    }
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            highest_ring: 0,
            islands_explored: 0,
            blocks_placed: 0,
            blocks_broken: 0,
            mobs_killed: 0,
            deaths: 0,
            void_deaths: 0,
            wind_deaths: 0,
            distance_walked: 0.0,
            play_time_ticks: 0,
        }
    }
}
