# Block Breaking & Mining

## General Rules
- Each block has hardness value, determines break time
- Break time = hardness * speed_multiplier (depends on tool)
- Wrong tool: break time x3.33, no item drop for most blocks
- Underwater: break time x5 (unless Aqua Affinity enchant)
- Not on ground (flying/falling): break time x5
- Both penalties stack: underwater + floating = x25

## Tool Tiers (speed multiplier)
- Hand: x1 (base)
- Wood: x2
- Stone: x4
- Iron: x6
- Diamond: x8
- Netherite: x9 (skip for now, no Nether)
- Gold: x12 (but very low durability)

## Common Block Break Times (seconds, with correct tool)

### Instant break (0 ticks)
- Torch, flower, tall grass, mushroom, crops (mature), fire, snow layer, lily pad

### Very fast (< 1 sec)
- Dirt/grass: 0.75s hand, 0.4s shovel
- Sand/gravel: 0.75s hand, 0.4s shovel
- Leaves: 0.35s shears, slow by hand
- Netherrack: 0.4s pickaxe
- Glowstone: 0.45s any tool
- Wool: 1.2s hand, 0.25s shears

### Medium (1-5 sec)
- Wood/log: 3s hand, 1.5s wood axe, 0.75s stone, 0.5s iron, 0.4s diamond
- Cobblestone: 10s hand, 1.5s wood pick, 0.75s stone, 0.5s iron, 0.4s diamond
- Stone: 7.5s hand, 1.15s wood pick, 0.6s stone, 0.4s iron, 0.3s diamond
- Sandstone: same as stone
- Bricks: same as cobblestone

### Slow (5+ sec)
- Iron ore: needs stone+ pick, 0.75s stone, 0.5s iron, 0.4s diamond
- Gold ore: needs iron+ pick, 0.75s iron, 0.55s diamond
- Diamond ore: needs iron+ pick, 0.75s iron, 0.55s diamond
- Iron block: needs stone+ pick, 1.5s stone, 0.75s iron, 0.6s diamond
- Obsidian: needs diamond pick, 9.4s diamond (250s by hand, no drop)

### Unbreakable
- Bedrock, barrier, end portal frame -- not applicable in Sky Craft
- We have void instead of bedrock

## Tool Durability
- Wood: 59 uses
- Stone: 131
- Iron: 250
- Diamond: 1561
- Gold: 32
- Shears: 238
- Bow: 384

## Drop Rules
- Most blocks drop themselves
- Stone drops cobblestone (silk touch = stone)
- Ores: coal ore->coal, iron ore->raw iron, gold ore->raw gold, diamond ore->diamond
- Grass block drops dirt (silk touch = grass block)
- Leaves: 1/20 sapling, 1/200 apple (oak), 1/20 stick
- Gravel: 10% flint drop chance
- Fortune enchant increases ore/flint/seed drops
- Silk Touch: drops the block itself instead of normal drop
