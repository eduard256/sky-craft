# World Generation (Sky Craft Islands)

## Overview
- World is infinite XZ plane of floating islands with void above and below
- Islands generated procedurally from seed (deterministic: same seed = same world)
- No bedrock, no underground caves (islands are solid chunks of terrain floating in void)
- Chunk-based: 16x16x16 cubic chunks, only non-empty chunks stored

## Island Placement
- Islands placed on a grid with noise-based offset for organic feel
- Base grid spacing: ~200-400 blocks between island centers (randomized with seed)
- Cluster algorithm: some areas have 3-5 nearby islands (50-100 apart), others isolated (300+)
- Y position: base Y=64, slight gradual rise as distance from world origin increases
- Y variance: most islands at similar height, some +/-10-20 blocks for visual interest
- Gap between closest edges: min 20 blocks, max ~300 blocks (bridgeable but challenging)

## Island Shape
- Top surface: Perlin noise terrain (hills, flat areas, small valleys)
- Bottom: inverted dome/cone shape tapering to a point (Avatar-style floating rocks)
- Vines/roots hanging from bottom edges
- Size range: 10-1000 blocks width/length, 10-150 blocks depth
- Size distribution: many small/medium (30-200), few large (500-1000)
- Shape influenced by biome: desert = flat mesa, forest = rounded hills, mountain = sharp peaks

## Biome Assignment
- Each island gets one biome (no multi-biome islands for simplicity)
- Biome selection: noise-based regional biomes (cluster nearby islands share similar biomes)
- Temperature/humidity gradient like MC: cold biomes cluster, warm cluster, etc

## Biome List (MC Overworld, adapted for islands)
- Plains: flat grass, flowers, scattered trees
- Forest: dense oak/birch trees
- Dark Forest: dense dark oak, giant mushrooms, low light
- Birch Forest: birch trees only
- Taiga: spruce trees, snow at high Y, wolves
- Snowy Plains: snow cover, ice, igloos, strays, polar bears
- Desert: sand, sandstone, cacti, dead bushes, husks, temples
- Savanna: acacia trees, dry grass, horses, llamas
- Jungle: tall jungle trees, vines, parrots, ocelots, melons, cocoa
- Swamp: shallow water, lily pads, clay, slimes, witch huts
- Badlands (Mesa): terracotta layers, red sand, gold ore boosted, no passive mobs
- Mushroom: mycelium, giant mushrooms, mooshrooms, no hostile spawns
- Flower Forest: dense flowers, bees
- Meadow: flowers, bee nests, occasional village
- Beach-like edges: sand border on island edges near water pools
- Mountain/Stony: stone surface, emerald ore, goats, snow peaks

## Surface Features
- Trees: placed per biome rules, oak/birch/spruce/jungle/acacia/dark oak
- Flowers/grass: density per biome
- Water pools: some islands have small lakes/ponds (water doesn't flow off edge, contained)
- Lava pools: rare, on large islands, contained in stone basin
- Sugar cane: near water on sand/dirt
- Pumpkins: rare surface patches
- Villages: on Plains/Desert/Savanna/Taiga/Snowy islands (large islands only, 300+ blocks)
- Pillager outposts: rare, on any biome large island
- Witch huts: swamp islands
- Dungeon spawners: inside larger islands (zombie/skeleton/spider spawner + 1-2 chests)

## Underground (within island mass)
- Ores distributed by depth within island:
  - Coal: anywhere, common
  - Iron: mid-depth, common
  - Gold: deep, uncommon
  - Diamond: deepest 16 blocks, rare
  - Lapis: mid-depth, uncommon
  - Emerald: mountain biome only, rare
  - Copper: mid-depth, common
- Small caves within large islands (not extensive cave systems)
- Mineshafts: rare, only in very large islands (500+)
- Dungeons: small rooms with spawner + chests

## Loot Chests (Sky Craft Special)
- Each island has 1 chest at random surface location
- Chest contains biome-specific loot:
  - Desert: gold, TNT, sand-related items
  - Forest: saplings, apples, wooden tools
  - Taiga: leather, furs, iron
  - Jungle: bamboo, cocoa, enchanted books
  - Swamp: potions, lily pads, slime balls
  - Mushroom: mushroom stew, mycelium
  - Plains: wheat, seeds, bread, basic tools
  - Mountain: emeralds, iron, stone tools
- Chest rarity tiers: common items always + chance for rare items
- One-time generation, doesn't respawn
