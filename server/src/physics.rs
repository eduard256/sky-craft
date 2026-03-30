// Physics simulation: gravity, collision, liquid flow, entity movement.

use skycraft_protocol::types::*;

/// Gravity acceleration in blocks/tick^2 (MC: 0.08 blocks/tick^2).
pub const GRAVITY: f64 = 0.08;

/// Air drag multiplier per tick.
pub const AIR_DRAG: f64 = 0.98;

/// Water drag multiplier per tick.
pub const WATER_DRAG: f64 = 0.8;

/// Terminal velocity in blocks/tick.
pub const TERMINAL_VELOCITY: f64 = 3.92;

/// Player walk speed in blocks/tick.
pub const WALK_SPEED: f64 = 0.2163; // 4.317 blocks/sec / 20 tps

/// Player sprint speed in blocks/tick.
pub const SPRINT_SPEED: f64 = 0.2806; // 5.612 blocks/sec / 20 tps

/// Player sneak speed in blocks/tick.
pub const SNEAK_SPEED: f64 = 0.0648;

/// Jump velocity in blocks/tick.
pub const JUMP_VELOCITY: f64 = 0.42;

/// Player hitbox dimensions.
pub const PLAYER_WIDTH: f64 = 0.6;
pub const PLAYER_HEIGHT: f64 = 1.8;
pub const PLAYER_EYE_HEIGHT: f64 = 1.62;

/// Void damage Y threshold.
pub const VOID_DAMAGE_Y: f64 = -64.0;

/// Instant kill Y threshold.
pub const VOID_KILL_Y: f64 = -128.0;

/// Apply gravity to a velocity, returning the new velocity.
pub fn apply_gravity(vel: Velocity, in_water: bool) -> Velocity {
    let drag = if in_water { WATER_DRAG } else { AIR_DRAG };
    let new_y = ((vel.y - GRAVITY) * drag).max(-TERMINAL_VELOCITY);
    Velocity {
        x: vel.x * drag,
        y: new_y,
        z: vel.z * drag,
    }
}

/// Calculate fall damage from fall distance in blocks.
/// Returns 0 if distance < 3 blocks.
pub fn fall_damage(fall_distance: f64) -> f32 {
    if fall_distance <= 3.0 {
        0.0
    } else {
        (fall_distance - 3.0) as f32
    }
}

/// Calculate wind push force on an entity.
/// Returns delta velocity to apply.
pub fn wind_push(wind: &WindState, is_sneaking: bool, has_wind_charm: bool, armor_weight: f32) -> Velocity {
    if wind.strength < 0.01 {
        return Velocity { x: 0.0, y: 0.0, z: 0.0 };
    }

    let dir_rad = (wind.direction as f64).to_radians();
    let mut strength = wind.strength as f64 / 20.0; // per tick

    if wind.gusting {
        strength *= 3.0;
    }

    // Reductions
    if is_sneaking {
        strength *= 0.3; // 70% reduction
    }
    if has_wind_charm {
        strength *= 0.5; // 50% reduction
    }
    // Iron+ armor reduces by 30%
    if armor_weight > 10.0 {
        strength *= 0.7;
    }

    Velocity {
        x: -dir_rad.sin() * strength,
        y: 0.0,
        z: dir_rad.cos() * strength,
    }
}

/// Simple AABB collision check between entity position and block grid.
/// Returns corrected position and whether entity is on ground.
pub struct CollisionResult {
    pub position: EntityPos,
    pub velocity: Velocity,
    pub on_ground: bool,
}

/// Check if a position collides with any solid block.
/// `is_solid` callback checks if a block position is solid.
pub fn move_and_collide(
    pos: EntityPos,
    vel: Velocity,
    width: f64,
    height: f64,
    is_solid: &dyn Fn(BlockPos) -> bool,
) -> CollisionResult {
    let half_w = width / 2.0;

    // Try to move, check each axis independently
    let mut new_pos = pos;
    let mut new_vel = vel;
    let mut on_ground = false;

    // Move Y axis first (gravity most important)
    let test_y = EntityPos::new(pos.x, pos.y + vel.y, pos.z);
    if !check_collision(test_y, half_w, height, is_solid) {
        new_pos.y = test_y.y;
    } else {
        if vel.y < 0.0 {
            // Hit ground
            on_ground = true;
            // Snap to block top
            new_pos.y = new_pos.y.floor() + 0.001;
        }
        new_vel.y = 0.0;
    }

    // Move X axis
    let test_x = EntityPos::new(new_pos.x + vel.x, new_pos.y, new_pos.z);
    if !check_collision(test_x, half_w, height, is_solid) {
        new_pos.x = test_x.x;
    } else {
        new_vel.x = 0.0;
    }

    // Move Z axis
    let test_z = EntityPos::new(new_pos.x, new_pos.y, new_pos.z + vel.z);
    if !check_collision(test_z, half_w, height, is_solid) {
        new_pos.z = test_z.z;
    } else {
        new_vel.z = 0.0;
    }

    CollisionResult {
        position: new_pos,
        velocity: new_vel,
        on_ground,
    }
}

/// Check if an entity AABB at a given position intersects any solid block.
fn check_collision(
    pos: EntityPos,
    half_width: f64,
    height: f64,
    is_solid: &dyn Fn(BlockPos) -> bool,
) -> bool {
    let min_x = (pos.x - half_width).floor() as i32;
    let max_x = (pos.x + half_width).floor() as i32;
    let min_y = pos.y.floor() as i32;
    let max_y = (pos.y + height).floor() as i32;
    let min_z = (pos.z - half_width).floor() as i32;
    let max_z = (pos.z + half_width).floor() as i32;

    for bx in min_x..=max_x {
        for by in min_y..=max_y {
            for bz in min_z..=max_z {
                if is_solid(BlockPos::new(bx, by, bz)) {
                    return true;
                }
            }
        }
    }
    false
}

/// Calculate the ring number for a position.
pub fn ring_at(x: f64, z: f64) -> u32 {
    let dist = (x * x + z * z).sqrt();
    (dist / 500.0) as u32
}

/// Calculate wind state for a given ring and world tick.
pub fn calculate_wind(ring: u32, world_tick: u64) -> WindState {
    if ring == 0 {
        return WindState {
            direction: 0.0,
            strength: 0.0,
            gusting: false,
        };
    }

    // Base wind strength increases with ring
    let base_strength = (ring as f32 * 0.3).min(15.0);

    // Direction changes over time (slow oscillation)
    let dir_period = 600.0 + (ring as f64 * 10.0); // ticks per full rotation
    let direction = ((world_tick as f64 / dir_period) * 360.0) % 360.0;

    // Gusts: random based on tick
    let gust_hash = (world_tick.wrapping_mul(2654435761) >> 20) % 100;
    let gust_chance = (ring as u64).min(30); // up to 30% chance per check
    let gusting = gust_hash < gust_chance;

    WindState {
        direction: direction as f32,
        strength: base_strength,
        gusting,
    }
}
