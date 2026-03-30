// Core data types shared between client and server.
// These types are the building blocks for all packets.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Positions ──────────────────────────────────────────────────────────────

/// World position of a block (integer coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Precise entity position (floating point).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EntityPos {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Entity rotation (yaw = horizontal, pitch = vertical).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rotation {
    /// Horizontal rotation in degrees (0-360). 0 = south, 90 = west.
    pub yaw: f32,
    /// Vertical rotation in degrees (-90 to 90). -90 = up, 90 = down.
    pub pitch: f32,
}

/// Velocity vector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Chunk position (chunk coordinates, NOT block coordinates).
/// Each chunk is 16x16x16 blocks. Chunk (0,0,0) covers blocks (0,0,0) to (15,15,15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Convert block position to the chunk that contains it.
    pub fn to_chunk_pos(&self) -> ChunkPos {
        ChunkPos {
            x: self.x.div_euclid(16),
            y: self.y.div_euclid(16),
            z: self.z.div_euclid(16),
        }
    }

    /// Position within the chunk (0-15 for each axis).
    pub fn chunk_local(&self) -> (u8, u8, u8) {
        (
            self.x.rem_euclid(16) as u8,
            self.y.rem_euclid(16) as u8,
            self.z.rem_euclid(16) as u8,
        )
    }
}

impl EntityPos {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Convert to block position (floor).
    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos {
            x: self.x.floor() as i32,
            y: self.y.floor() as i32,
            z: self.z.floor() as i32,
        }
    }

    /// Distance to another position.
    pub fn distance_to(&self, other: &EntityPos) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Horizontal distance (XZ plane only, ignores Y). Used for ring calculation.
    pub fn horizontal_distance_to_origin(&self) -> f64 {
        (self.x * self.x + self.z * self.z).sqrt()
    }
}

impl ChunkPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

// ─── Block State ────────────────────────────────────────────────────────────

/// Block state ID. Maps to a specific block + variant (e.g. oak_log facing north).
/// 0 = air. IDs match common/data/blocks.json state IDs.
pub type BlockStateId = u16;

/// Block light level (0-15).
pub type LightLevel = u8;

// ─── Items ──────────────────────────────────────────────────────────────────

/// Item type ID. Maps to common/data/items.json.
pub type ItemId = u16;

/// A stack of items in an inventory slot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemStack {
    pub item_id: ItemId,
    pub count: u8,
    /// Remaining durability. None for non-durability items.
    pub durability: Option<u16>,
    /// Enchantments on this item. List of (enchantment_id, level).
    pub enchantments: Vec<(u16, u8)>,
    /// Custom display name (from anvil rename). None if default.
    pub custom_name: Option<String>,
}

/// An inventory slot: either empty or containing an item stack.
pub type Slot = Option<ItemStack>;

// ─── Entities ───────────────────────────────────────────────────────────────

/// Unique entity ID, assigned by server per session. Not persistent across restarts.
pub type EntityId = u32;

/// Entity type ID. Maps to common/data/entities.json.
pub type EntityTypeId = u16;

/// Player UUID. Persistent across sessions, assigned during auth registration.
pub type PlayerId = Uuid;

// ─── Chunk Data ─────────────────────────────────────────────────────────────

/// Chunk section: 16x16x16 block states + light data.
/// Block states stored as palette + indices for compact representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSection {
    /// Palette: list of unique block state IDs in this section.
    /// If palette has 1 entry, entire section is that single block (e.g. all air).
    pub palette: Vec<BlockStateId>,

    /// Block indices into the palette. Length = 4096 (16*16*16).
    /// Each value is an index into `palette`.
    /// Stored in YZX order: index = y*256 + z*16 + x.
    /// Empty if palette has exactly 1 entry (uniform section).
    pub blocks: Vec<u16>,

    /// Block light levels. Length = 4096 or empty (all 0).
    /// Each value is 0-15. Same YZX ordering as blocks.
    pub block_light: Vec<u8>,

    /// Sky light levels. Length = 4096 or empty (all 15).
    /// Each value is 0-15. Same YZX ordering as blocks.
    pub sky_light: Vec<u8>,
}

impl ChunkSection {
    /// Number of blocks per section axis.
    pub const SIZE: usize = 16;

    /// Total blocks in a section.
    pub const VOLUME: usize = Self::SIZE * Self::SIZE * Self::SIZE;

    /// Create an empty (all air) section.
    pub fn empty() -> Self {
        Self {
            palette: vec![0], // 0 = air
            blocks: vec![],   // empty = uniform (all palette[0])
            block_light: vec![],
            sky_light: vec![],
        }
    }

    /// Check if section is entirely air.
    pub fn is_empty(&self) -> bool {
        self.palette.len() == 1 && self.palette[0] == 0
    }

    /// Get block state at local position (0-15 each axis).
    pub fn get_block(&self, x: u8, y: u8, z: u8) -> BlockStateId {
        if self.palette.len() == 1 {
            return self.palette[0];
        }
        let index = (y as usize) * 256 + (z as usize) * 16 + (x as usize);
        let palette_index = self.blocks[index] as usize;
        self.palette[palette_index]
    }
}

// ─── Game State ─────────────────────────────────────────────────────────────

/// Difficulty levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

/// Game modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Survival,
    Creative,
    Spectator,
}

/// Cardinal direction for block face interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockFace {
    Bottom, // -Y
    Top,    // +Y
    North,  // -Z
    South,  // +Z
    West,   // -X
    East,   // +X
}

/// Hand (for item use, block placement).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hand {
    Main,
    Off,
}

/// Player action states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerAction {
    StartDigging,
    CancelDigging,
    FinishDigging,
    DropItem,
    DropItemStack,
    UseItem,
    SwapHands,
}

/// Chat message type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatType {
    /// Regular player chat: <nickname> message
    Player,
    /// System message (server announcements, death messages, etc.)
    System,
    /// Private message from another player.
    Whisper,
}

// ─── Sky Craft Specific ─────────────────────────────────────────────────────

/// Ring number (concentric zone around world origin).
pub type RingNumber = u32;

/// Active debuff applied by mobs at high rings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobDebuff {
    /// Cannot place blocks. Duration in ticks.
    PlacementLock(u32),
    /// Cannot break blocks. Duration in ticks.
    MiningLock(u32),
    /// Cannot open inventory/chests. Duration in ticks.
    InventoryLock(u32),
    /// Pulled toward nearest void edge. Strength in blocks/tick.
    GravityPull(u32),
    /// Screen shake, vision distortion. Duration in ticks.
    Fear(u32),
    /// Nausea + slowness when below island. Duration in ticks.
    VoidSickness(u32),
    /// Lose XP levels on hit. Amount of levels drained.
    SoulDrain(u8),
    /// Bed spawn point destroyed remotely.
    AnchorBreak,
}

/// Potion effect with ID and metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PotionEffect {
    /// Effect type ID from common/data/effects.json.
    pub effect_id: u8,
    /// Amplifier (0 = level I, 1 = level II, etc).
    pub amplifier: u8,
    /// Remaining duration in ticks.
    pub duration: u32,
    /// Whether particles are visible.
    pub show_particles: bool,
}

/// Wind state sent to client for rendering and physics.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindState {
    /// Wind direction in degrees (0-360, like yaw). 0 = south, 90 = west.
    pub direction: f32,
    /// Wind strength in blocks per second.
    pub strength: f32,
    /// Whether a gust is currently active.
    pub gusting: bool,
}

/// Island metadata sent to client when entering a new island.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IslandInfo {
    /// Procedurally generated island name.
    pub name: String,
    /// Biome name (e.g. "Forest", "Desert").
    pub biome: String,
    /// Approximate island size (width x length in blocks).
    pub size_x: u16,
    pub size_z: u16,
    /// Ring this island belongs to.
    pub ring: RingNumber,
}

/// Death cause for death screen message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeathCause {
    /// Killed by entity. Contains entity type name and ring number.
    EntityKill { entity_name: String, ring: RingNumber },
    /// Fell into void.
    VoidFall,
    /// Blown off by wind.
    WindBlown,
    /// Struck by void lightning.
    VoidLightning,
    /// Fall damage.
    FallDamage,
    /// Drowning.
    Drowning,
    /// Fire/lava.
    Fire,
    /// Starvation.
    Starvation,
    /// Explosion.
    Explosion,
    /// Killed by player.
    PlayerKill { killer: String },
    /// Generic/other.
    Other { message: String },
}

/// Achievement/milestone notification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Achievement {
    /// Title shown to player.
    pub title: String,
    /// Description shown below title.
    pub description: String,
}
