# Mob Spawning Rules (Sky Craft Adapted)

## General Rules (Same as MC)
- Mob cap per player: 70 hostile, 10 passive, 15 ambient, 5 water
- Hostile spawns: every tick, within 24-128 blocks of player
- Passive spawns: during world gen + rarely after, on grass at light 9+
- Despawn: >128 blocks = instant, 32-128 = random chance, <32 = never
- Named mobs (name tag): never despawn
- Spawn surface: solid opaque block, 2 blocks air above (most mobs)

## Sky Craft Adaptations

### Void Edge Behavior
- Mobs CAN walk off island edges (no invisible barriers)
- Mobs that fall into void die at Y < -64 (same as player)
- This naturally thins mob population on small islands
- AI pathfinding avoids edges (3 block buffer) but isn't perfect
- Creepers chasing player WILL follow off edges
- Skeletons strafe and may fall off during combat

### Island Surface Spawning
- Mobs only spawn on island surfaces and player-built structures
- Cannot spawn in void (no solid blocks)
- Cannot spawn on cloud platforms
- Bridge surfaces: valid spawn location if dark and wide enough
- 1-block-wide bridges: too narrow for most mobs (need 1+ wide for zombies/skeletons)
- 2-block-wide bridges: all mobs can spawn
- Slabs on bridges: prevent mob spawning (standard MC slab rule)

### Ring-Based Spawn Rates
```
Ring 0:  1.0x spawn rate (standard MC)
Ring 1:  1.1x
Ring 2:  1.2x
Ring 3:  1.3x
Ring 5:  1.5x
Ring 10: 2.0x
Ring 20: 3.0x
Ring 50: 5.0x
Ring 100: 8.0x
```
- Higher spawn rate = more mobs per spawning cycle
- Combined with stronger mobs = exponentially harder

### Ring-Based Mob Selection
Each ring unlocks additional mob types and changes weights:

```
Ring 0: zombie (40%), skeleton (30%), spider (20%), creeper (10%)
Ring 1: + witch (5%), drowned (5%). Rebalance others proportionally
Ring 2: + enderman (3%), phantom (2%)
Ring 3: + pillager (3%), cave spider (2%)
Ring 5: + blaze (2%, volcanic only), wither skeleton (1%, volcanic only)
Ring 8: + vindicator (2%), evoker (1%)
Ring 10: + magma cube (2%, volcanic), ghast (1%, void between islands)
Ring 15: + ravager (0.5%), vex (summoned by evokers only)
Ring 20: + elder guardian (0.1%, water islands only)
```

### Ghast Special Spawning (Ring 10+)
- Ghasts don't spawn on islands
- Spawn in void air between islands
- Float between islands, shoot fireballs at players on bridges
- Only spawn when player is on bridge or in void
- Despawn when player returns to island surface
- Ghast fireballs can destroy bridge blocks (major threat)

### Phantom Adaptation
- Standard MC: spawn after 3 days without sleep
- Sky Craft: spawn after 2 days without sleep at ring 0, 1 day at ring 5+
- Ring scaling: phantoms at ring 10+ have double HP and deal fire damage
- Sleep in bed to reset phantom timer (same as MC)

### Day Spawning at High Rings
- Ring 10+: sky darkening allows some hostile spawns during day
- Ring 20+: full hostile spawning during day (sky too dark for light suppression)
- Ring 50+: no difference between day and night for spawning
- Only light from torches/blocks prevents spawning at high rings

### Passive Mob Island Assignment
- Passive mobs assigned during island generation
- Each island spawns 0-8 passive mobs on surface during gen
- Type based on biome:
  - Plains: cows (30%), sheep (30%), pigs (20%), horses (10%), chickens (10%)
  - Forest: pigs (30%), wolves (20%), chickens (20%), foxes (15%), rabbits (15%)
  - Taiga: wolves (25%), foxes (25%), rabbits (25%), sheep (15%), chickens (10%)
  - Desert: rabbits (80%), no other passive mobs
  - Jungle: parrots (30%), ocelots (25%), pandas (20%), chickens (15%), pigs (10%)
  - Swamp: frogs (50%), no other passive mobs naturally (must bring)
  - Mountain: goats (60%), sheep (20%), llamas (20%)
  - Savanna: horses (25%), donkeys (20%), llamas (20%), cows (15%), sheep (10%), chickens (10%)
  - Snowy: polar bears (30%), rabbits (40%), foxes (30%)
  - Mushroom: mooshrooms (100%)
- If all passive mobs on an island die: no natural respawn (must breed or transport from elsewhere)

### Mob Transport Between Islands
- Lead mobs with leads across bridges
- Push mobs into boats, sail across water or carry boat
- Chickens can be lured with seeds across bridges
- Breeding animals on outpost islands: important mid-game strategy
- Mobs can fall off bridges, so transport is risky
- At high rings: wind can push mobs off bridges (leashed mobs resist wind by 50%)

## Spawner Mechanics
- Found in dungeons within large islands
- Spawner types: zombie (50%), skeleton (25%), spider (25%)
- Spawner spawns mobs within 4-block radius, needs dark, every 10-40 sec
- Spawner can be disabled with torches (light level 8+)
- Spawner block cannot be obtained (breaks with no drop, even with silk touch)
- XP farms buildable around spawners (standard MC mob grinder designs)
- At high rings: spawner mobs inherit ring stat multipliers (stronger)
