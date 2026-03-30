# TODO: Village Generation & Trading

## Status: Not implemented

## What exists
- `island.rs`: has_village flag set during island generation for large Plains/Desert/Savanna/Taiga
- `unique_logic/villages.md`: full design spec

## What needs to be built

### Village structure generation
- Triggered during island generation for islands with has_village=true
- Place buildings from templates (hardcoded or schematic files)
- 3-10 buildings depending on island size
- Building types: house (bed), farm (crops+composter), well, smithy, library, church, butcher
- Architecture varies by biome (oak for plains, sandstone for desert, spruce for taiga)
- Roads: path blocks connecting buildings
- Central meeting point with bell
- Iron golem: spawn if 10+ villagers + 21+ beds

### Building templates
- Define as 3D block arrays (schematic format)
- Each template: list of (relative_pos, block_state) pairs
- Rotate templates randomly for variety
- Check placement area is clear before placing
- Small house: ~5x5x4 interior
- Large house: ~7x7x5
- Farm: 9x9 fenced area with water channel + crops
- Well: 3x3 with water source

### Villager entity spawning
- 1-3 villagers per house (based on beds)
- Random profession assignment (based on nearby workstation)
- Villager entity type with profession, level, trades data

### Trading system
- Right-click villager: open trade window
- Each profession has trade pool (from MC wiki data)
- 5 levels: Novice -> Master, XP gained by trading
- Supply/demand: prices change based on trade volume
- Reputation: curing zombie villager = discount, killing villager = price increase
- Key trades to implement first:
  - Farmer: wheat/carrot/potato for emeralds, bread for emeralds
  - Librarian: emeralds for enchanted books (critical for progression)
  - Toolsmith: emeralds for diamond tools at Master

### Villager AI
- Daytime: wander near workstation, occasionally use it
- Night: walk to bed, sleep
- Flee from hostile mobs (run inside house)
- Breed when enough beds + food surplus
- Gossip system (simplified): track player actions affecting prices
- Zombie villager conversion: on death by zombie, become zombie villager
- Zombie villager curing: weakness potion + golden apple -> villager with discount

### Raids
- Triggered by player with Bad Omen entering village
- Bad Omen: killing pillager captain
- 3-7 waves of illagers (pillagers, vindicators, evokers, witches, ravagers)
- Wave composition scales with difficulty
- Victory: Hero of the Village effect (reduced prices 2 game days)
- Defeat: villagers may die

### Files to create
- `server/src/world/village.rs` -- structure generation
- `server/src/world/templates.rs` -- building schematics
- `server/src/entity/villager.rs` -- villager AI, trading
- `server/src/entity/raid.rs` -- raid system

### Estimated: ~2000 lines
