# TODO: Block Logic

## Status: Not implemented

## What exists
- `world/generator.rs`: blocks placed during generation (surface, ores, stone)
- `game.rs`: break_block() sets to air, handle_block_place() sets block state
- No per-block-type logic

## What needs to be built

### Water flow
- Source block: stays, creates flow blocks in 4 horizontal directions
- Flow distance: 7 blocks on flat, unlimited vertical
- Flow direction: shortest path to nearest edge/drop
- Recalculate flow when source/neighbor placed or removed
- Infinite source: 2+ adjacent sources flowing into empty = new source
- Water + lava interactions (obsidian, cobblestone, stone)
- Waterlogged blocks (slabs, stairs, fences hold water)
- Waterfall off island edge: source stays, flow renders downward
- Push entities in flow direction

### Lava flow
- Same as water but 3 blocks horizontal, slower (30 ticks per spread vs 5)
- Lava off island edge: source IS consumed (drains)
- Fire ignition: lava ignites nearby flammable blocks
- Lava + water interactions
- No infinite source trick (unlike water)

### Gravity blocks
- Sand, gravel, red sand, concrete powder, anvil
- Check block below on placement and when neighbor changes
- If no block below: convert to falling entity
- Falling entity: gravity physics, lands on first solid block
- If lands on non-solid (torch, slab): drop as item
- If falls into void: destroyed

### Fire
- Ignition: flint&steel, lava nearby, lightning, fire spread
- Spread: check flammable neighbors every tick, random chance
- Spread distance: 1 block horizontal, 4 blocks down
- Burn out: 3-40 sec depending on block below
- Extinguish: water, rain, player (left-click fire block)
- Eternal fire: on netherrack, soul sand
- fire_tick gamerule: disable spread entirely

### Crop growth
- Random tick: each crop block has chance to grow each game tick
- Growth rate: affected by light, hydration, nearby crops
- Wheat/carrot/potato/beetroot: 8 stages, break for harvest
- Melon/pumpkin: stem grows, then spawns block on adjacent dirt
- Sugar cane: grows up to 3 tall on sand/dirt near water
- Cactus: grows up to 3 tall on sand, no adjacent blocks
- Bamboo: grows up to 16 tall, very fast
- Cocoa: grows on jungle log sides, 3 stages
- Trees: sapling + bone meal or random tick, space check above

### Farmland
- Created by hoeing dirt/grass
- Hydrated: water within 4 blocks (check radius)
- Trampled: reverts to dirt when jumped on / mob walks on
- Dehydrated: reverts to dirt if no water and no crop for too long

### Redstone (postponed to V2, but need block placement)
- Redstone dust, torch, repeater, comparator: place but no signal propagation
- Lever, button, pressure plate: place but no signal
- Piston, sticky piston: place but no push logic
- Door/trapdoor: manual open/close only (no redstone signal)

### Block interaction (right-click)
- Door/trapdoor/fence gate: toggle open/close
- Chest/barrel: open container window
- Furnace/blast furnace/smoker: open container window
- Crafting table: open 3x3 crafting window
- Enchanting table: open enchanting window
- Anvil: open anvil window
- Brewing stand: open brewing window
- Bed: sleep attempt (check night, check mobs nearby)
- Note block: play note (different pitch based on block below)
- Jukebox: insert/eject music disc
- Campfire: place food to cook (4 slots, 30 sec each)
- Cauldron: fill/empty with bucket, dye leather armor
- Composter: insert plant items, output bone meal
- Bell: ring animation + sound
- Sign: open text editor (place only, text set on creation)
- Beehive/bee nest: harvest with shears (honeycomb) or bottle (honey), smoke with campfire

### Block update propagation
- When block placed/removed: notify 6 adjacent blocks
- Adjacent blocks check validity (torch needs wall, crops need farmland, etc)
- Invalid blocks break and drop as items
- Chain reaction: gravity blocks, water flow recalculation
- Performance: limit updates per tick to prevent lag from large cascades

### Lighting recalculation
- When block placed: recalculate light in affected area
- Block light: BFS flood fill from light sources (torch=14, glowstone=15, etc)
- Sky light: top-down propagation, reduced by 1 per opaque block
- Update light in chunk section, send to clients
- Performance: batch light updates, process async

### Files to create
- `server/src/block/mod.rs` -- block update dispatcher
- `server/src/block/liquid.rs` -- water/lava flow
- `server/src/block/gravity.rs` -- falling blocks
- `server/src/block/fire.rs` -- fire spread/extinguish
- `server/src/block/farming.rs` -- crop growth, farmland
- `server/src/block/interact.rs` -- right-click block actions
- `server/src/block/lighting.rs` -- light propagation

### Estimated: ~5000 lines
