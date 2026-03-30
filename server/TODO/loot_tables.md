# TODO: Loot Tables

## Status: Not implemented

## What exists
- `entity.rs`: LootDrop struct (item_id, count range, chance, looting bonus)
- `game.rs`: kill entity removes it but no drops

## What needs to be built

### Mob drops
Define loot table per mob type. On death: roll each entry.
- Zombie: 0-2 rotten flesh, rare iron ingot/carrot/potato
- Skeleton: 0-2 bones, 0-2 arrows
- Creeper: 0-2 gunpowder, music disc if killed by skeleton
- Spider: 0-2 string, 0-1 spider eye
- Enderman: 0-1 ender pearl
- Cow: 1-3 leather, 1-3 raw beef
- Pig: 1-3 raw porkchop
- Sheep: 1 wool (color), 1-2 mutton
- Chicken: 1-2 feathers, 1 raw chicken
- Witch: 0-6 random (glowstone, sugar, redstone, spider eye, glass bottle, stick)
- Slime: 0-2 slimeballs (small only)
- Blaze: 0-1 blaze rod
- Ghast: 0-1 ghast tear, 0-2 gunpowder
- Guardian: 0-2 prismarine shards, 0-1 raw cod
- Iron golem: 3-5 iron ingots, 0-2 poppies
- All mobs: XP orbs (1-20 depending on type)
- Looting enchant: +1 max count per level
- Fire Aspect: drop cooked variant of meat

### Block drops
On break: determine drop based on block type + tool.
- Stone -> cobblestone (silk touch -> stone)
- Grass block -> dirt (silk touch -> grass)
- Ores -> respective item (silk touch -> ore block)
- Fortune: multiply ore drops (1-4x at Fortune III)
- Leaves: 1/20 sapling, 1/200 apple (oak), 1/20 stick
- Gravel: 10% flint
- Tall grass: ~10% seeds
- Wrong tool: no drop (most ores need specific pickaxe tier)
- Glass: no drop (silk touch -> glass)

### Chest loot
Generated when chest first opened (or during island generation).
- Biome-specific loot (see unique_logic/island_generation.md)
- Ring-scaled: higher ring = better loot
- Loot pool: list of (item, count_range, weight, min_ring)
- Roll 3-8 items per chest from pool

### XP orb spawning
- On mob death: spawn XP orb entity at death location
- XP value: mob-specific (zombie=5, blaze=10, etc)
- On ore mine: spawn XP orb (coal=0-2, diamond=3-7, etc)
- On smelting: accumulated XP released when taking output
- XP orbs merge nearby, fly toward nearest player within 8 blocks

### Data format
- Load from JSON files in common/data/ or hardcode
- Each entry: { mob/block type, drops: [{ item_id, min, max, chance, conditions }] }

### Files to create
- `server/src/loot/mod.rs` -- loot table registry
- `server/src/loot/mob_drops.rs` -- per-mob drop tables
- `server/src/loot/block_drops.rs` -- per-block drop tables
- `server/src/loot/chest_loot.rs` -- chest generation loot pools

### Estimated: ~1500 lines
