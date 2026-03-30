// Entity system. Mobs, items, projectiles -- everything that isn't a block.

use skycraft_protocol::types::*;
use serde::{Deserialize, Serialize};

/// Server-side entity data.
pub struct Entity {
    pub id: EntityId,
    pub entity_type: EntityTypeId,
    pub position: EntityPos,
    pub rotation: Rotation,
    pub velocity: Velocity,
    pub on_ground: bool,

    pub health: f32,
    pub max_health: f32,
    pub is_dead: bool,
    pub is_on_fire: bool,
    pub fire_ticks: u32,
    pub no_damage_ticks: u32,

    pub kind: EntityKind,
}

/// Specific entity data depending on type.
pub enum EntityKind {
    Mob(MobData),
    Item(ItemEntityData),
    Projectile(ProjectileData),
    ExperienceOrb(XpOrbData),
}

pub struct MobData {
    pub ai: MobAI,
    pub attack_damage: f32,
    pub attack_cooldown: u32,
    pub ticks_since_attack: u32,
    pub target: Option<EntityId>,
    pub home_pos: Option<BlockPos>,
    pub drops: Vec<LootDrop>,
    pub xp_reward: u32,
    pub ring_multiplier: f32,
    pub debuffs_on_hit: Vec<MobDebuff>,
    pub mob_category: MobCategory,
    pub despawn_timer: u32,
    pub is_baby: bool,
    pub can_pick_up_items: bool,
    pub equipment: [Slot; 6],
}

pub struct ItemEntityData {
    pub item: ItemStack,
    pub pickup_delay: u32,
    pub age: u32,
    pub owner: Option<PlayerId>,
}

pub struct ProjectileData {
    pub shooter: Option<EntityId>,
    pub damage: f32,
    pub projectile_type: ProjectileType,
    pub ticks_alive: u32,
}

pub struct XpOrbData {
    pub xp_value: u32,
    pub age: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileType {
    Arrow,
    Trident,
    Snowball,
    Egg,
    EnderPearl,
    FishingBobber,
    Fireball,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobCategory {
    Hostile,
    Passive,
    Neutral,
    Ambient,
    Water,
}

/// Mob AI state machine.
pub struct MobAI {
    pub state: AIState,
    pub wander_target: Option<BlockPos>,
    pub wander_cooldown: u32,
    pub look_target: Option<EntityPos>,
    pub path: Vec<BlockPos>,
    pub path_index: usize,
    pub stuck_ticks: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIState {
    Idle,
    Wandering,
    Chasing,
    Attacking,
    Fleeing,
    Breeding,
    Eating,
    Sleeping,
    Panicking,
}

/// Loot drop definition (item + chance + count range).
pub struct LootDrop {
    pub item_id: ItemId,
    pub min_count: u8,
    pub max_count: u8,
    pub chance: f32,
    pub looting_bonus: f32,
}

impl Entity {
    /// Create a new mob entity.
    pub fn new_mob(
        id: EntityId,
        entity_type: EntityTypeId,
        position: EntityPos,
        health: f32,
        damage: f32,
        ring_multiplier: f32,
    ) -> Self {
        let scaled_health = health * ring_multiplier;
        let scaled_damage = damage * ring_multiplier;

        Self {
            id,
            entity_type,
            position,
            rotation: Rotation { yaw: 0.0, pitch: 0.0 },
            velocity: Velocity { x: 0.0, y: 0.0, z: 0.0 },
            on_ground: true,
            health: scaled_health,
            max_health: scaled_health,
            is_dead: false,
            is_on_fire: false,
            fire_ticks: 0,
            no_damage_ticks: 0,
            kind: EntityKind::Mob(MobData {
                ai: MobAI {
                    state: AIState::Idle,
                    wander_target: None,
                    wander_cooldown: 0,
                    look_target: None,
                    path: Vec::new(),
                    path_index: 0,
                    stuck_ticks: 0,
                },
                attack_damage: scaled_damage,
                attack_cooldown: 20,
                ticks_since_attack: 0,
                target: None,
                home_pos: None,
                drops: Vec::new(),
                xp_reward: 5,
                ring_multiplier,
                debuffs_on_hit: Vec::new(),
                mob_category: MobCategory::Hostile,
                despawn_timer: 0,
                is_baby: false,
                can_pick_up_items: false,
                equipment: Default::default(),
            }),
        }
    }

    /// Create a dropped item entity.
    pub fn new_item(id: EntityId, position: EntityPos, item: ItemStack, velocity: Velocity) -> Self {
        Self {
            id,
            entity_type: 0, // item entity type
            position,
            rotation: Rotation { yaw: 0.0, pitch: 0.0 },
            velocity,
            on_ground: false,
            health: 5.0,
            max_health: 5.0,
            is_dead: false,
            is_on_fire: false,
            fire_ticks: 0,
            no_damage_ticks: 0,
            kind: EntityKind::Item(ItemEntityData {
                item,
                pickup_delay: 40, // 2 seconds
                age: 0,
                owner: None,
            }),
        }
    }

    /// Create a projectile entity.
    pub fn new_projectile(
        id: EntityId,
        position: EntityPos,
        velocity: Velocity,
        shooter: Option<EntityId>,
        damage: f32,
        projectile_type: ProjectileType,
    ) -> Self {
        Self {
            id,
            entity_type: 0,
            position,
            rotation: Rotation { yaw: 0.0, pitch: 0.0 },
            velocity,
            on_ground: false,
            health: 1.0,
            max_health: 1.0,
            is_dead: false,
            is_on_fire: false,
            fire_ticks: 0,
            no_damage_ticks: 0,
            kind: EntityKind::Projectile(ProjectileData {
                shooter,
                damage,
                projectile_type,
                ticks_alive: 0,
            }),
        }
    }

    /// Apply damage to entity. Returns true if entity died.
    pub fn take_damage(&mut self, amount: f32) -> bool {
        if self.no_damage_ticks > 0 {
            return false;
        }
        self.health = (self.health - amount).max(0.0);
        self.no_damage_ticks = 10;
        if self.health <= 0.0 {
            self.is_dead = true;
            true
        } else {
            false
        }
    }
}

impl Default for MobAI {
    fn default() -> Self {
        Self {
            state: AIState::Idle,
            wander_target: None,
            wander_cooldown: 0,
            look_target: None,
            path: Vec::new(),
            path_index: 0,
            stuck_ticks: 0,
        }
    }
}
