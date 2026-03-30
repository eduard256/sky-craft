# TODO: Sky Craft Unique Mechanics

## Status: Partially implemented (wind calculation exists, rest is stubs/missing)

## What exists
- `physics.rs`: calculate_wind() function
- `game.rs`: tick_wind() sends WindUpdate to players, ring detection
- `player.rs`: active_debuffs vec, has_debuff() check
- Protocol: all Sky Craft packets defined (RingUpdate, WindUpdate, DebuffApplied, HazardWarning, AuroraEvent, etc)
- `unique_logic/` folder: full design specs for everything

## What needs to be built

### Wind physics (partially done)
- Apply wind push to players on bridges (not on solid island surface)
- Check "open air" condition: no solid block within 3 blocks horizontally
- Sneaking reduces push by 70%
- Wind charm item: reduces by 50%
- Iron+ armor: reduces by 30%
- Push dropped items and arrows mid-flight
- Push mobs on bridges (mobs can fall off too)
- Wind gusts: 0.5 sec HUD warning before gust hits

### Void lightning (ring 10+)
- During thunderstorms: strike random void locations every 30-90 sec
- Ring 30+: 20% chance targeting nearest player on bridge
- Ring 50+: 50% targeting chance
- Damage: 8 HP + fire 5 sec
- Destroys 1-3 bridge blocks at strike location
- 1 sec warning: HazardWarning packet + crackling sound
- Lightning rod block attracts strikes, protects nearby blocks

### Void fog (ring 5+)
- Random event: 2-5 min duration, every 10-30 min
- Send HazardWarning(VoidFog) to affected players
- Client reduces render distance during fog
- Ring 5-10: 5 chunks visibility
- Ring 30+: 1-2 chunks visibility
- Mobs spawned during fog: 50% chance Invisibility effect

### Falling debris (ring 15+)
- Spawn falling block entities above islands
- 1-3 per minute per loaded chunk
- Block types: cobblestone, stone, gravel, sand
- Damage on hit: 3-8 HP
- Shadow warning on ground 1 sec before impact

### Island tremors (ring 25+)
- Random tremor every 5-15 min on occupied islands
- Duration: 2-5 seconds
- Camera shake: send particle/sound effect to client
- Gravity blocks (sand, gravel) may fall during tremor
- Slowness I on all entities during tremor

### Aurora event (ring 5+)
- Random event: every 2-3 game days, unpredictable
- Duration: 5-10 min
- Send AuroraEvent(active=true) to all players
- During aurora: all hostile mobs become passive (stop attacking)
- Wind calms to zero
- Mob debuffs suspended
- Perfect window for bridge building
- 30 sec warning before end: AuroraEvent with low remaining_secs
- On end: mobs immediately return to hostile

### Mob debuffs (ring 3+)
- On mob melee hit: check ring, roll for debuff application
- PlacementLock: prevent block placement for N seconds
- MiningLock: prevent block breaking for N seconds
- InventoryLock: prevent opening inventory for N seconds
- GravityPull: push player toward nearest island edge
- Fear: send screen shake effect (particle + sound)
- VoidSickness: apply Nausea + Slowness when below island
- SoulDrain: reduce player XP levels
- AnchorBreak: destroy player's bed spawn point
- Send DebuffApplied packet on application
- Send DebuffExpired packet when timer expires
- Debuff assignment by ring: see unique_logic/mob_buffs.md

### Bridge decay (ring 15+)
- Track last-visited timestamp per chunk
- If no player within 5 chunks for 7+ real days:
  - Wood blocks: 2-3 blocks removed per day
  - Stone/cobble: 1 block per 7 days
  - Obsidian: never decays
  - Dirt/sand: 5+ blocks per day
- Island Anchor block: prevents decay within 200 block radius

### Sky fishing
- Detect fishing bobber in void (below island bottom Y)
- Different loot table than water fishing
- Loot: string, feathers, phantom membrane, glowstone dust, prismarine, enchanted books, name tags, saddles
- Bite time: 15-45 sec (longer than water)

### Grappling hook physics (partially done)
- Verify target block is solid and within 20 blocks
- Apply velocity toward target (0.4 blocks/tick = 8 blocks/sec)
- Consume 1 durability from grappling hook item
- Cancel if player takes damage
- Cannot use during PlacementLock debuff

### Cloud platforms
- Generate semi-solid cloud blocks in void between islands (ring 2+)
- Noise-based placement, deterministic with seed
- Player can stand for 5 sec, then sinks through (1 block/sec)
- Reset after player leaves for 10 sec
- Mobs fall through immediately
- Cannot place blocks on clouds

### Updrafts
- Vertical air columns between some islands (ring 3+)
- Generated deterministically with seed
- 3-5 blocks diameter
- Push player upward at 5 blocks/sec
- Particle effect: swirling white column

### Void wells
- Rare downward vortexes (ring 5+)
- Pull radius: 8 blocks, strength: 2-4 blocks/sec
- Blocks placed inside slowly destroyed (1 per 30 sec)
- Must build bridges around, not through
- Humming sound audible from 20 blocks

### Resonance mining
- On hitting ore block: nearby same-type ores glow for 3 sec
- Radius: 5 blocks
- Send particle effect to hitting player only
- Glow visible through stone blocks

### Island naming
- Calculate name from seed + island coordinates (deterministic)
- Format: "[Adjective] [Biome] [Suffix]"
- Send name in IslandInfo when player enters island
- Already implemented in island.rs: island_name() function

### Custom crafting recipes
- Register Sky Craft recipes alongside MC recipes
- Void Crystal Shard: found item, not craftable
- Void Compass: compass + 4 void crystal shards
- Grappling Hook: 3 iron + 1 tripwire hook + 2 string
- Island Anchor: 4 obsidian + 4 iron blocks + 1 diamond
- Sky Lantern: 5 paper + 1 torch = 4
- Bridge Rail: 6 iron + 1 stick = 16
- Weather Station: glass + 2 iron + clock + compass + 2 redstone
- Emergency Recall: 4 ender pearls + 4 gold ingots + 1 bed
- Wind Charm: 4 feathers + 4 string + 1 ender pearl
- Void Binoculars: 2 glass + 1 gold + 2 iron

### Void Resistance Potion
- Brewing: awkward potion + void crystal shard
- Effect: immune to void damage for 30 sec
- Redstone modifier: 60 sec
- Splash variant with gunpowder

### Sky darkening (ring 10+)
- Reduce sky light level based on ring
- Ring 10: sky light -2
- Ring 20: sky light -4, hostile spawning during day possible
- Ring 50: sky light -8, permanent dusk
- Ring 100+: permanent night sky
- Affects spawn rules and visibility

### Files to create
- `server/src/skycraft/mod.rs` -- unique mechanics dispatcher
- `server/src/skycraft/wind.rs` -- wind push application to entities
- `server/src/skycraft/hazards.rs` -- void lightning, fog, debris, tremors
- `server/src/skycraft/aurora.rs` -- aurora event logic
- `server/src/skycraft/debuffs.rs` -- mob debuff application/ticking
- `server/src/skycraft/bridge_decay.rs` -- bridge decay timer
- `server/src/skycraft/sky_fishing.rs` -- void fishing loot
- `server/src/skycraft/clouds.rs` -- cloud platforms
- `server/src/skycraft/updrafts.rs` -- vertical air columns
- `server/src/skycraft/void_wells.rs` -- downward vortexes
- `server/src/skycraft/recipes.rs` -- custom crafting recipes

### Estimated: ~3000 lines
