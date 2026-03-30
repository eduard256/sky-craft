// All packet definitions for client-server communication.
//
// Naming convention: C2S_ = client to server, S2C_ = server to client.
// Each packet has a unique ID used for routing.
//
// Connection phases:
//   1. Login    -- auth handshake (bi-directional on a dedicated QUIC stream)
//   2. Play     -- game packets (multiple QUIC streams for different data channels)
//
// QUIC streams:
//   - Stream 0 (bidirectional): login phase, then control packets (keep-alive, disconnect)
//   - Stream 1 (server->client): chunk data (bulk, can lag without blocking controls)
//   - Stream 2 (bidirectional): game actions (movement, block changes, combat)
//   - Stream 3 (bidirectional): chat, UI interactions, inventory

use serde::{Deserialize, Serialize};
use crate::types::*;

// ─── Packet Wrapper ─────────────────────────────────────────────────────────

/// Top-level packet sent from client to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientPacket {
    // ── Login Phase ──
    /// First packet. Client sends auth token and protocol version.
    Login(C2SLogin),

    // ── Play Phase: Movement ──
    /// Player position update. Sent every tick when moving.
    PlayerPosition(C2SPlayerPosition),
    /// Player look direction update. Sent when camera moves.
    PlayerLook(C2SPlayerLook),
    /// Combined position + look update (most common, saves bandwidth).
    PlayerPositionAndLook(C2SPlayerPositionAndLook),

    // ── Play Phase: Actions ──
    /// Player action (start/stop digging, drop item, use item, swap hands).
    PlayerAction(C2SPlayerAction),
    /// Place a block.
    BlockPlace(C2SBlockPlace),
    /// Start or finish breaking a block.
    BlockDig(C2SBlockDig),
    /// Interact with entity (attack or right-click).
    EntityInteract(C2SEntityInteract),
    /// Use held item (eat, throw, shoot bow).
    UseItem(C2SUseItem),
    /// Swing arm animation.
    SwingArm(C2SSwingArm),

    // ── Play Phase: Inventory ──
    /// Click on inventory/container slot.
    ClickSlot(C2SClickSlot),
    /// Change held hotbar slot (0-8).
    HeldItemChange(C2SHeldItemChange),
    /// Close currently open container/inventory screen.
    CloseWindow(C2SCloseWindow),
    /// Creative mode: set slot contents.
    CreativeSetSlot(C2SCreativeSetSlot),

    // ── Play Phase: Communication ──
    /// Chat message or command.
    ChatMessage(C2SChatMessage),

    // ── Play Phase: Control ──
    /// Response to server's keep-alive.
    KeepAliveResponse(C2SKeepAliveResponse),
    /// Client settings (view distance, etc).
    ClientSettings(C2SClientSettings),

    // ── Play Phase: Sky Craft Specific ──
    /// Place a bridge marker.
    PlaceMarker(C2SPlaceMarker),
    /// Remove a bridge marker.
    RemoveMarker(C2SRemoveMarker),
    /// Use grappling hook.
    UseGrapplingHook(C2SUseGrapplingHook),
    /// Use emergency recall.
    UseEmergencyRecall,
}

/// Top-level packet sent from server to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerPacket {
    // ── Login Phase ──
    /// Login accepted. Contains player info.
    LoginSuccess(S2CLoginSuccess),
    /// Login rejected. Contains reason.
    Disconnect(S2CDisconnect),

    // ── Play Phase: World ──
    /// Full chunk section data.
    ChunkData(S2CChunkData),
    /// Unload a chunk (player moved away).
    UnloadChunk(S2CUnloadChunk),
    /// Single block change within a loaded chunk.
    BlockChange(S2CBlockChange),
    /// Multiple block changes in one chunk (batch update).
    MultiBlockChange(S2CMultiBlockChange),

    // ── Play Phase: Entities ──
    /// Spawn a new entity (mob, item, projectile).
    SpawnEntity(S2CSpawnEntity),
    /// Spawn a player entity (other players).
    SpawnPlayer(S2CSpawnPlayer),
    /// Entity moved (delta position).
    EntityMove(S2CEntityMove),
    /// Entity rotation changed.
    EntityLook(S2CEntityLook),
    /// Entity moved and rotated.
    EntityMoveAndLook(S2CEntityMoveAndLook),
    /// Entity teleported (absolute position, for large moves).
    EntityTeleport(S2CEntityTeleport),
    /// Entity velocity changed (knockback, explosion push).
    EntityVelocity(S2CEntityVelocity),
    /// Entity removed from world.
    DestroyEntities(S2CDestroyEntities),
    /// Entity metadata update (HP, status effects, armor, held item).
    EntityMetadata(S2CEntityMetadata),
    /// Entity animation (swing arm, take damage, etc).
    EntityAnimation(S2CEntityAnimation),
    /// Entity equipment change (armor, held item visible to others).
    EntityEquipment(S2CEntityEquipment),

    // ── Play Phase: Player State ──
    /// Update player health, hunger, saturation.
    UpdateHealth(S2CUpdateHealth),
    /// Update player XP bar and level.
    SetExperience(S2CSetExperience),
    /// Set player position (server correction, teleport, respawn).
    PlayerPositionAndLook(S2CPlayerPositionAndLook),
    /// Respawn after death (new dimension data, clear state).
    Respawn(S2CRespawn),

    // ── Play Phase: Inventory ──
    /// Set contents of a window (inventory, chest, crafting table, etc).
    WindowItems(S2CWindowItems),
    /// Set a single slot in a window.
    SetSlot(S2CSetSlot),
    /// Open a container window (chest, furnace, enchanting table, etc).
    OpenWindow(S2COpenWindow),
    /// Confirm a slot click transaction.
    ConfirmTransaction(S2CConfirmTransaction),

    // ── Play Phase: Communication ──
    /// Chat message from player or system.
    ChatMessage(S2CChatMessage),

    // ── Play Phase: World State ──
    /// Time of day update (world age + time).
    TimeUpdate(S2CTimeUpdate),
    /// Weather change.
    WeatherChange(S2CWeatherChange),
    /// Sound effect at position.
    SoundEffect(S2CSoundEffect),
    /// Particle effect at position.
    ParticleEffect(S2CParticleEffect),
    /// Explosion (TNT, creeper).
    Explosion(S2CExplosion),

    // ── Play Phase: Player List ──
    /// Add/remove/update players in tab list.
    PlayerListUpdate(S2CPlayerListUpdate),

    // ── Play Phase: Control ──
    /// Keep-alive ping. Client must respond with KeepAliveResponse.
    KeepAlive(S2CKeepAlive),

    // ── Play Phase: Sky Craft Specific ──
    /// Current ring number and island info.
    RingUpdate(S2CRingUpdate),
    /// Wind state update.
    WindUpdate(S2CWindUpdate),
    /// Mob debuff applied to player.
    DebuffApplied(S2CDebuffApplied),
    /// Mob debuff expired.
    DebuffExpired(S2CDebuffExpired),
    /// Potion effect applied/updated.
    EffectApplied(S2CEffectApplied),
    /// Potion effect expired.
    EffectExpired(S2CEffectExpired),
    /// Environmental hazard warning (void lightning, fog, tremor).
    HazardWarning(S2CHazardWarning),
    /// Aurora event started/ended.
    AuroraEvent(S2CAuroraEvent),
    /// Death screen info (cause, coordinates).
    DeathInfo(S2CDeathInfo),
    /// Achievement/milestone notification.
    AchievementUnlocked(S2CAchievementUnlocked),
    /// Sky fishing catch result.
    SkyFishingCatch(S2CSkyFishingCatch),
}

// ─── Client -> Server Packet Structs ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SLogin {
    pub protocol_version: u32,
    pub auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SPlayerPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SPlayerLook {
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SPlayerPositionAndLook {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SPlayerAction {
    pub action: PlayerAction,
    pub position: BlockPos,
    pub face: BlockFace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SBlockPlace {
    pub hand: Hand,
    pub position: BlockPos,
    pub face: BlockFace,
    /// Cursor position on the block face (0.0-1.0 for each axis).
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub cursor_z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SBlockDig {
    pub action: DiggingAction,
    pub position: BlockPos,
    pub face: BlockFace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiggingAction {
    Start,
    Cancel,
    Finish,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SEntityInteract {
    pub entity_id: EntityId,
    pub action: EntityInteractAction,
    pub hand: Hand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityInteractAction {
    Attack,
    Interact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SUseItem {
    pub hand: Hand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SSwingArm {
    pub hand: Hand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SClickSlot {
    /// Window ID (0 = player inventory, others = open container).
    pub window_id: u8,
    /// Slot index clicked.
    pub slot: i16,
    /// Mouse button (0 = left, 1 = right, 2 = middle).
    pub button: u8,
    /// Click mode (normal, shift-click, number key, drop, drag).
    pub mode: ClickMode,
    /// Expected slot contents after click (for server validation).
    pub clicked_item: Slot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClickMode {
    Normal,
    ShiftClick,
    NumberKey(u8),
    Drop,
    DoubleClick,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SHeldItemChange {
    /// Hotbar slot index (0-8).
    pub slot: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SCloseWindow {
    pub window_id: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SCreativeSetSlot {
    pub slot: i16,
    pub item: Slot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SChatMessage {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SKeepAliveResponse {
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SClientSettings {
    pub view_distance: u8,
    pub chat_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SPlaceMarker {
    pub position: BlockPos,
    /// Marker color index (0-7).
    pub color: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SRemoveMarker {
    pub position: BlockPos,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2SUseGrapplingHook {
    pub target: BlockPos,
}

// ─── Server -> Client Packet Structs ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CLoginSuccess {
    pub player_uuid: PlayerId,
    pub nickname: String,
    pub game_mode: GameMode,
    pub difficulty: Difficulty,
    pub spawn_position: EntityPos,
    pub world_seed: i64,
    pub view_distance: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CDisconnect {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CChunkData {
    pub chunk_pos: ChunkPos,
    pub section: ChunkSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CUnloadChunk {
    pub chunk_pos: ChunkPos,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CBlockChange {
    pub position: BlockPos,
    pub block_state: BlockStateId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CMultiBlockChange {
    pub chunk_pos: ChunkPos,
    /// List of (local_x, local_y, local_z, new_block_state).
    pub changes: Vec<(u8, u8, u8, BlockStateId)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CSpawnEntity {
    pub entity_id: EntityId,
    pub entity_type: EntityTypeId,
    pub position: EntityPos,
    pub rotation: Rotation,
    pub velocity: Velocity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CSpawnPlayer {
    pub entity_id: EntityId,
    pub player_uuid: PlayerId,
    pub nickname: String,
    pub position: EntityPos,
    pub rotation: Rotation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CEntityMove {
    pub entity_id: EntityId,
    /// Delta position in 1/4096 of a block (high precision, compact).
    pub dx: i16,
    pub dy: i16,
    pub dz: i16,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CEntityLook {
    pub entity_id: EntityId,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CEntityMoveAndLook {
    pub entity_id: EntityId,
    pub dx: i16,
    pub dy: i16,
    pub dz: i16,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CEntityTeleport {
    pub entity_id: EntityId,
    pub position: EntityPos,
    pub rotation: Rotation,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CEntityVelocity {
    pub entity_id: EntityId,
    pub velocity: Velocity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CDestroyEntities {
    pub entity_ids: Vec<EntityId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CEntityMetadata {
    pub entity_id: EntityId,
    pub health: Option<f32>,
    pub custom_name: Option<String>,
    pub is_on_fire: bool,
    pub is_sneaking: bool,
    pub is_sprinting: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationType {
    SwingMainArm,
    TakeDamage,
    LeaveBed,
    SwingOffhand,
    CriticalEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CEntityAnimation {
    pub entity_id: EntityId,
    pub animation: AnimationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CEntityEquipment {
    pub entity_id: EntityId,
    /// Equipment slot: 0=main hand, 1=off hand, 2=boots, 3=legs, 4=chest, 5=helmet.
    pub slot: u8,
    pub item: Slot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CUpdateHealth {
    pub health: f32,
    pub food: u8,
    pub saturation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CSetExperience {
    /// XP bar progress (0.0 to 1.0).
    pub bar: f32,
    /// Current level.
    pub level: u16,
    /// Total XP points.
    pub total_xp: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CPlayerPositionAndLook {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CRespawn {
    pub game_mode: GameMode,
    pub difficulty: Difficulty,
    pub spawn_position: EntityPos,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CWindowItems {
    pub window_id: u8,
    pub slots: Vec<Slot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CSetSlot {
    pub window_id: u8,
    pub slot: i16,
    pub item: Slot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2COpenWindow {
    pub window_id: u8,
    pub window_type: WindowType,
    pub title: String,
    /// Number of slots in the container (not counting player inventory).
    pub slot_count: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowType {
    Chest,
    DoubleChest,
    CraftingTable,
    Furnace,
    BlastFurnace,
    Smoker,
    Anvil,
    EnchantingTable,
    BrewingStand,
    Barrel,
    ShulkerBox,
    Grindstone,
    Stonecutter,
    Loom,
    CartographyTable,
    SmithingTable,
    Beacon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CConfirmTransaction {
    pub window_id: u8,
    pub accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CChatMessage {
    pub message: String,
    pub sender: Option<String>,
    pub chat_type: ChatType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CTimeUpdate {
    /// Total world age in ticks (always increasing).
    pub world_age: u64,
    /// Time of day in ticks (0-24000). 0=sunrise, 6000=noon, 12000=sunset, 18000=midnight.
    pub time_of_day: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weather {
    Clear,
    Rain,
    Thunder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CWeatherChange {
    pub weather: Weather,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CSoundEffect {
    /// Sound ID (mapped to sound file name by client).
    pub sound_id: u16,
    pub position: EntityPos,
    /// Volume (0.0 to 1.0).
    pub volume: f32,
    /// Pitch multiplier (0.5 to 2.0, 1.0 = normal).
    pub pitch: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CParticleEffect {
    /// Particle type ID.
    pub particle_id: u16,
    pub position: EntityPos,
    /// Offset/spread for particle spawn area.
    pub offset_x: f32,
    pub offset_y: f32,
    pub offset_z: f32,
    /// Number of particles.
    pub count: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CExplosion {
    pub position: EntityPos,
    /// Explosion radius.
    pub radius: f32,
    /// Blocks destroyed by explosion (relative to position).
    pub destroyed_blocks: Vec<(i8, i8, i8)>,
    /// Velocity applied to nearby player.
    pub player_velocity: Velocity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CPlayerListUpdate {
    pub action: PlayerListAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerListAction {
    /// Add player to list.
    Add {
        uuid: PlayerId,
        nickname: String,
        game_mode: GameMode,
        ping_ms: u16,
    },
    /// Update player's ping.
    UpdatePing { uuid: PlayerId, ping_ms: u16 },
    /// Remove player from list.
    Remove { uuid: PlayerId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CKeepAlive {
    pub id: u64,
}

// ── Sky Craft Specific Packets ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CRingUpdate {
    /// Current ring number.
    pub ring: RingNumber,
    /// Island info if player is standing on an island, None if in void/on bridge.
    pub island: Option<IslandInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CWindUpdate {
    pub wind: WindState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CDebuffApplied {
    pub debuff: MobDebuff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CDebuffExpired {
    pub debuff_type: DebuffType,
}

/// Debuff type enum (without duration data, just the type for expiration messages).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebuffType {
    PlacementLock,
    MiningLock,
    InventoryLock,
    GravityPull,
    Fear,
    VoidSickness,
    SoulDrain,
    AnchorBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CEffectApplied {
    pub effect: PotionEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CEffectExpired {
    pub effect_id: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CHazardWarning {
    pub hazard: HazardType,
    pub position: Option<EntityPos>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HazardType {
    VoidLightning,
    VoidFog,
    FallingDebris,
    IslandTremor,
    WindGust,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CAuroraEvent {
    /// true = aurora started, false = aurora ended.
    pub active: bool,
    /// Seconds remaining (0 if ending).
    pub remaining_secs: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CDeathInfo {
    pub cause: DeathCause,
    /// Position where player died.
    pub death_position: EntityPos,
    /// XP score on death.
    pub score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CAchievementUnlocked {
    pub achievement: Achievement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2CSkyFishingCatch {
    pub item: ItemStack,
}
