# Island Generation

## Seed System
- World seed: 64-bit integer, can be set manually or generated randomly
- Same seed = identical world (deterministic generation)
- Seed displayed in server config and via /seed command
- Shareable: players can copy seed to recreate same world on another server

## Island Placement Algorithm

### Grid-Based with Noise Offset
1. Divide world XZ into cells of 400x400 blocks
2. Each cell has 0-3 islands (determined by seed + cell coords hash)
3. Island center position within cell: offset by Perlin noise (so not grid-aligned)
4. Additional clustering: some cells marked as "archipelago" by large-scale noise,
   these cells get 2-3 islands close together (40-80 blocks apart)
5. Isolated cells get 0-1 islands with larger gaps to neighbors

### Island Spacing
- Minimum gap between island edges: 20 blocks (guaranteed)
- Typical gap ring 0: 30-80 blocks
- Gap increases with ring number: base_gap + ring * 2
- Maximum gap: 500 blocks (even at extreme rings)

## Island Shape Generation

### Top Surface (Terrain)
1. Generate heightmap using 2D Perlin noise (octaves: 4, persistence: 0.5)
2. Scale height by island size: small islands = flatter, large islands = more varied
3. Biome determines terrain style:
   - Plains: gentle rolling hills, max height variation 8 blocks
   - Forest: moderate hills, 12 blocks variation
   - Desert: flat mesa tops with sharp edges, 4 blocks variation
   - Mountain/Stony: sharp peaks, 20+ blocks variation
   - Jungle: hilly with steep slopes, 15 blocks variation
   - Swamp: very flat, 3 blocks variation, lots of water
   - Mushroom: rounded dome shape
4. Apply biome-specific surface blocks (grass, sand, mycelium, etc)

### Bottom Surface (Floating Rock)
- Avatar-style inverted mountain shape
- Generated using 3D noise carving from a solid block mass
- Tapers toward a rough point at the bottom (not perfectly pointed)
- Wider at top where it meets the terrain, narrowing downward
- Depth: 10-150 blocks below the terrain surface
- Surface material: stone with patches of mossy cobblestone, andesite, diorite
- Vines and roots hang from overhangs and the underside
- Occasional dripping water particles from bottom
- Small stalactite-like stone formations on underside

### Bottom Shape Algorithm
```
For each column (x, z) on the island:
  terrain_y = top surface height at (x, z)
  center_dist = distance from (x, z) to island center, normalized 0-1

  // Base taper: deeper in center, shallower at edges
  base_depth = max_depth * (1.0 - center_dist^0.7)

  // Add noise for organic shape
  noise_offset = perlin3d(x*0.05, terrain_y*0.05, z*0.05) * max_depth * 0.3

  bottom_y = terrain_y - base_depth + noise_offset

  // Ensure minimum 3 blocks of solid ground at edges
  bottom_y = min(bottom_y, terrain_y - 3)

  // Fill column from bottom_y to terrain_y with appropriate blocks
```

### Edge Profile
- Edges are NOT flat vertical cuts
- Overhang frequency: 30% of edge blocks have 1-3 block overhangs
- Grass/dirt extends 2-4 blocks down from terrain surface at edges
- Then transitions to stone/deepslate
- Occasional exposed ore veins visible on cliff faces (decorative + minable)
- Small ledges on sides where mobs can stand (and fall off)

## Island Interior (Underground)

### Layer Structure (top to bottom within island)
```
Surface:    grass/sand/biome-specific (1-3 blocks)
Topsoil:    dirt (3-5 blocks)
Stone:      stone, andesite, granite, diorite (bulk of island)
Deep stone: deepslate (bottom 30% of island depth, only on islands 50+ blocks deep)
```

### Ore Distribution
Follows MC logic but adapted for island depth instead of world Y:

```
depth_in_island = distance from surface going down

Coal:     depth 0-100%, peak at 20%, density = 17 veins per chunk
Iron:     depth 20-80%, peak at 50%, density = 12 veins per chunk
Copper:   depth 10-60%, peak at 30%, density = 6 veins per chunk
Lapis:    depth 40-80%, peak at 60%, density = 2 veins per chunk (ring 1+)
Gold:     depth 50-90%, peak at 75%, density = 3 veins per chunk (ring 1+)
Redstone: depth 60-100%, peak at 85%, density = 4 veins per chunk (ring 2+)
Diamond:  depth 80-100%, peak at 95%, density = 1 vein per chunk (ring 2+)
Emerald:  depth 50-100%, single blocks, mountain biome only (ring 2+)
```

Ore density multiplied by ring progression multiplier (see rings_progression.md).

### Vein Size
- Coal: 5-17 blocks per vein
- Iron: 4-9 blocks
- Copper: 5-10 blocks
- Gold: 4-9 blocks
- Lapis: 3-7 blocks
- Redstone: 4-8 blocks
- Diamond: 1-8 blocks (smaller veins more common)
- Emerald: always single blocks

### Caves Within Islands
- Only islands 80+ blocks deep have internal caves
- Cave frequency: 1-3 small caves per large island
- Cave size: 5-20 blocks long tunnels, 3-5 blocks diameter
- Caves can expose ores on walls
- No massive cave systems (islands aren't big enough)
- Cave generation: 3D worm noise, seeded per island

### Dungeons
- Islands 100+ blocks wide and 50+ deep: 10% chance of dungeon
- Dungeon: 5x5x4 room with spawner (zombie/skeleton/spider) + 1-2 chests
- Spawner type weighted by ring: ring 0-2 zombie heavy, ring 3+ balanced
- Loot tables scale with ring number

## Biome-Specific Features

### Plains Island
- Flat terrain with gentle hills
- Tall grass, flowers (dandelion, poppy, cornflower)
- 30% chance of village on islands 300+ wide
- 1-3 oak trees per 100 blocks^2
- Occasional small pond (10-20 blocks diameter)

### Forest Island
- Dense oak and birch trees (5-8 per 100 blocks^2)
- Thick undergrowth (tall grass, ferns)
- Mushrooms under dense canopy
- Wolves spawn here
- Bee nests in ~5% of oak trees

### Dark Forest Island
- Very dense dark oak trees, canopy blocks most sky light
- Giant mushrooms between trees
- Hostile mobs can spawn during day under canopy
- Higher mob density than other biomes
- Rare woodland mansion structure on 500+ wide islands (ring 3+)

### Birch Forest Island
- All birch trees, lighter feel
- More flowers than regular forest
- Slightly less mob spawning (brighter floor)

### Taiga Island
- Spruce trees, sweet berry bushes
- Snow at Y > 90 (or on high-ring islands)
- Wolves, foxes, rabbits
- Podzol surface blocks under spruce
- Villages with spruce wood architecture

### Snowy Plains Island
- Snow layer on all surfaces
- Ice on water surfaces
- Igloos (5% of medium+ islands)
- Strays instead of skeletons
- Polar bears
- Snow golems don't take damage here

### Desert Island
- Sand surface (5-8 blocks deep), sandstone below
- Cacti, dead bushes
- No passive mobs except rabbits
- Husks instead of zombies
- Desert temple on large islands (200+): 4 chests + TNT trap
- No water pools (desert islands are dry)

### Savanna Island
- Acacia trees, coarse dirt patches
- Horses, donkeys, llamas
- Dry grass texture
- Villages with acacia wood
- Flat-topped terrain

### Jungle Island
- Tall jungle trees (up to 30 blocks), vines everywhere
- Bamboo patches, melons on ground, cocoa on tree trunks
- Parrots, ocelots, pandas (near bamboo)
- Dense undergrowth limits visibility
- Jungle temple on large islands (300+): puzzles + traps + chests

### Swamp Island
- Very flat, lots of small water pools
- Oak trees with vines hanging from leaves
- Lily pads on water, clay under water
- Witch huts (small stilt house, witch spawns inside)
- Slimes spawn at surface during night
- Blue orchid flowers

### Badlands (Mesa) Island
- Terracotta layers in bands of color (orange, yellow, red, brown, white)
- Red sand surface
- Gold ore at all depths (not just deep)
- No passive mob spawning
- Mineshaft structures more common (exposed on surface in some cases)

### Mushroom Island
- Mycelium surface
- Giant red and brown mushrooms instead of trees
- Mooshrooms instead of cows
- NO hostile mob spawning (safe zone)
- Extremely rare biome (1 in 50 islands)
- Very valuable as safe outpost location

### Mountain/Stony Island
- Stone surface with patches of gravel and grass
- Emerald ore exclusive to this biome
- Goats
- Snow above certain height
- Dramatic cliff faces, lots of exposed stone
- Good for mining (high ore density in exposed faces)

## Water Features

### Ponds and Lakes
- 40% of islands have at least one water feature
- Small pond: 5-15 blocks diameter, 2-4 blocks deep
- Lake: 20-50 blocks diameter, 3-8 blocks deep (only on 200+ wide islands)
- Water contained within island (natural basin)
- Sugar cane grows at water edges
- Fish (cod, salmon) spawn in lakes
- Clay under water bottom

### Waterfalls
- When water source is near island edge, water flows off edge
- Creates a waterfall into the void (visual effect)
- Source block is NOT consumed (infinite waterfall)
- Waterfall renders as falling water stream that fades out after 20 blocks
- Players can ride waterfall down (take no fall damage while in water)
- Climbing back up waterfall: possible but slow (swimming up mechanic)

### Island Edge Water
- Some islands have small water streams that naturally flow to edges
- Creates permanent waterfalls around the island perimeter
- Aesthetic + gameplay: can descend to underside of island via waterfall

## Lava Features
- Lava pools: rare, only on islands 150+ wide, ring 1+
- Always contained in stone basin (doesn't flow off edge by default)
- If player breaks containment: lava flows to edge and creates lava-fall
- Lava source IS consumed if it flows off edge (not infinite like water)
- Lava pools on volcanic-themed islands (see special islands below)

## Special Island Types

### Volcanic Island (Ring 3+)
- Netherrack, basalt, blackstone composition
- Central lava lake (15-30 blocks diameter)
- Magma blocks on surface
- Blaze spawners in small obsidian structures (1-2 per island)
- Wither skeletons patrol surface
- Nether wart grows on soul sand patches
- Critical for brewing progression (blaze rods, nether wart)
- Fire resistant mobs only

### Ore Pillar (Ring 20+)
- Tiny island (5-15 blocks wide), tall pillar shape
- Made almost entirely of ore blocks
- Very little surface area, hard to build on
- Usually no mobs (too small for spawning)
- High risk: easy to fall off while mining

### Jackpot Island (Ring 100+)
- Small to medium island (20-100 blocks)
- Surface and interior contain precious blocks (gold, diamond, emerald blocks)
- Swarming with extreme-level mobs
- Worth an entire expedition to raid
- Players need to fight or distract mobs to mine

### Cloud Island (Ring 5+)
- Made of snow, packed ice, blue ice
- Very slippery surface
- No trees, no ores (only ice and snow)
- Snow golems spawn naturally
- Contains ice-variant resources
- Beautiful but resource-poor

### Floating Ruin (Ring 4+)
- Small island (20-50 blocks) with crumbling stone brick structure
- Resembles a destroyed tower or building
- Contains 2-4 loot chests with mid-to-high tier loot
- Silverfish infested stone blocks in walls
- One spawner (random hostile mob)
- No natural ores but good loot

### Garden Island (Ring 2+)
- Lush vegetation: all flower types, bee nests, berry bushes
- Moss blocks, dripleaf, azalea trees
- Passive mobs only (no hostile spawning, like mushroom island)
- Rare biome (1 in 40 islands)
- Good for farming, breeding, safe outpost

## Island Naming
- Each island gets a procedural name for map/HUD display
- Format: [Adjective] [Biome] [Suffix]
- Adjectives: Verdant, Barren, Windy, Misty, Ancient, Forgotten, Frozen, Burning, etc
- Suffixes: Isle, Crest, Rock, Peak, Haven, Reef, Spire, etc
- Example: "Misty Forest Haven", "Ancient Desert Spire", "Frozen Taiga Rock"
- Name generated deterministically from seed + island coordinates
