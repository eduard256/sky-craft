# TODO: World Structure Generation

## Status: Not implemented (islands are flat terrain + ores only)

## What exists
- `island.rs`: IslandDef with biome, has_village, has_loot_chest flags
- `generator.rs`: surface block, topsoil, stone layers, ores, precious blocks
- No trees, no structures, no water pools, no decorations

## What needs to be built

### Trees
- Place during chunk generation on grass/dirt surface blocks
- Tree types by biome:
  - Oak: plains, forest, swamp (vine variant)
  - Birch: birch forest, flower forest
  - Spruce: taiga, snowy (snow on leaves)
  - Jungle: jungle (tall variant + vines)
  - Acacia: savanna (diagonal trunk)
  - Dark oak: dark forest (2x2 trunk, dense canopy)
- Tree structure: trunk (log blocks) + canopy (leaf blocks)
- Small tree: 5-7 blocks tall, 3-5 block radius canopy
- Large tree: 10-20 blocks tall (jungle, dark oak)
- Placement: check space above (no overlap with terrain/other trees)
- Density by biome: forest=8/chunk, plains=1-2/chunk, desert=0
- Bee nests: 5% of oak/birch trees

### Flowers and grass
- Tall grass/fern: scatter on grass blocks, density by biome
- Flowers: dandelion, poppy, cornflower, allium, etc
- Flower forest: extremely dense flowers
- Swamp: blue orchid, lily pads on water
- Meadow: mixed flowers

### Water pools
- 40% of islands have small pond (5-15 blocks diameter, 2-4 deep)
- Larger islands: lake (20-50 blocks, 3-8 deep)
- Water contained in natural basin (don't flow off edge)
- Sugar cane at water edges on sand/dirt
- Clay under water bottom
- Fish entities spawn in lakes

### Lava pools
- Rare, only on islands 150+ wide, ring 1+
- Contained in stone basin
- Size: 5-15 blocks diameter
- More common on volcanic islands

### Dungeons
- Islands 100+ wide, 50+ deep: 10% chance
- 5x5x4 room: mossy cobblestone + cobblestone walls
- 1 spawner (zombie 50%, skeleton 25%, spider 25%)
- 1-2 chests with loot

### Desert temple
- Large desert islands (200+)
- Sandstone structure with colored terracotta
- 4 chests in hidden basement + TNT trap

### Jungle temple
- Large jungle islands (300+)
- Cobblestone/mossy cobblestone structure
- Puzzle mechanisms (levers), trap (dispensers with arrows)
- 2 chests

### Witch hut
- Swamp islands
- Small stilt house (oak planks + spruce planks)
- Witch spawns inside
- 1 chest

### Pillager outpost
- Any biome, large islands, ring 2+
- Dark oak structure, banner on top
- Pillager captain with banner
- Loot chest

### Floating ruin (Sky Craft unique)
- Special island type (biome 17)
- Crumbling stone brick structure
- Infested stone blocks (silverfish)
- 2-4 loot chests
- 1 spawner

### Loot chest placement
- Every island: 1 chest at random surface location
- Buried 1 block into ground or in small stone niche
- Biome-specific loot table (see unique_logic/island_generation.md)

### Volcanic island structures (ring 3+)
- Obsidian pillars with blaze spawner
- Soul sand patches with nether wart
- Magma block paths
- Basalt/blackstone composition

### Files to create
- `server/src/world/structures/mod.rs` -- structure placement dispatcher
- `server/src/world/structures/trees.rs` -- tree generation
- `server/src/world/structures/decoration.rs` -- flowers, grass, water pools
- `server/src/world/structures/dungeon.rs` -- dungeons
- `server/src/world/structures/temple.rs` -- desert/jungle temples
- `server/src/world/structures/village.rs` -- (see villages.md)
- `server/src/world/structures/volcanic.rs` -- volcanic island features

### Estimated: ~3000 lines
