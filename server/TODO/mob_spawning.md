# TODO: Mob Spawning System

## Status: Not implemented (mobs only exist if manually created)

## What exists
- `entity.rs`: Entity::new_mob() constructor with ring multiplier
- `game.rs`: entity tick loop, despawn timer
- `unique_logic/mob_spawning_rules.md`: full design spec

## What needs to be built

### Spawn cycle (every tick for hostile, every 400 ticks for passive)
- For each player: find valid spawn positions within 24-128 block radius
- Check mob cap: 70 hostile, 10 passive, 15 ambient, 5 water per player
- Count existing mobs in range toward cap
- If below cap: attempt spawn

### Spawn position validation
- Find random block in range around player
- Check: solid opaque block below, 2 blocks air above (most mobs)
- Check: light level 0 for hostile (block light, not sky light)
- Check: not within 24 blocks of any player
- Check: on island surface (not in void)
- Bridge spawning: only if bridge >= 2 blocks wide and dark

### Mob type selection by ring
- Ring 0: zombie 40%, skeleton 30%, spider 20%, creeper 10%
- Ring 1+: add witch, drowned, adjust weights
- Ring 2+: add enderman, phantom
- Ring 3+: add pillager, cave spider, blaze (volcanic)
- Ring 5+: add wither skeleton (volcanic)
- Ring 10+: add ghast (void spawning), magma cube
- Full table in unique_logic/mob_spawning_rules.md

### Ring stat scaling
- HP: base * (1.0 + ring * 0.2)
- Damage: base * (1.0 + ring * 0.25)
- Speed bonus: min(ring * 0.02, 1.0)
- Group size: min(1 + floor(ring / 10), 20)
- Debuff assignment: see unique_logic/mob_buffs.md

### Passive mob spawning
- During island generation: place 0-8 passive mobs on grass surfaces
- Biome determines types (cows in plains, wolves in forest, etc)
- Once spawned: persist until killed, no natural respawn
- Player must breed or transport from other islands

### Ghast special spawning (ring 10+)
- Don't spawn on islands
- Spawn in void air when player is on bridge
- Float between islands, shoot fireballs at bridges
- Despawn when player returns to island

### Phantom spawning
- Timer: ticks_without_sleep per player
- Ring 0: spawn after 3 days (72000 ticks)
- Ring 5+: spawn after 1 day (24000 ticks)
- Spawn 1-3 phantoms above player in sky
- Sleep in bed resets timer

### Day spawning at high rings
- Ring 10+: sky darkening reduces effective sky light
- Some hostile mobs can spawn during day
- Ring 20+: full hostile spawning during day

### Files to create
- `server/src/spawn/mod.rs` -- spawn cycle orchestrator
- `server/src/spawn/hostile.rs` -- hostile mob spawn logic
- `server/src/spawn/passive.rs` -- passive mob placement during worldgen
- `server/src/spawn/special.rs` -- ghast, phantom, ring-specific spawning

### Estimated: ~1500 lines
