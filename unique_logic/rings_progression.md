# Ring Progression System

## Overview
The world is divided into concentric rings around the world origin (0,0). Each ring is harder
and more rewarding than the previous one. There is no final ring -- difficulty and rewards
scale infinitely. The ring number determines everything: mob strength, ore density, block
composition, loot quality, environmental hazards.

## Ring Calculation
- Ring 0: center, radius 0-500 blocks from origin (XZ distance)
- Ring 1: 500-1000 blocks
- Ring 2: 1000-1500
- Ring N: N*500 to (N+1)*500 blocks from 0,0
- Formula: ring = floor(sqrt(x^2 + z^2) / 500)
- Ring number displayed on HUD at all times (see hud.md)

## Ring Height
- Each ring is slightly higher than the previous one
- Ring 0: base Y=64
- Ring N: base Y = 64 + N*3 (capped at Y=320)
- Islands within a ring vary by +/-10 blocks from ring base Y
- This means building bridges between rings sometimes requires going uphill

## Ring 0 -- Starter Zone
- Biomes: Plains, Forest, Birch Forest, Flower Forest, Meadow
- Islands: medium size (100-400 blocks wide)
- Ores: coal (abundant), iron (common), copper (common)
- No gold, no diamond, no emerald, no lapis
- Mobs: standard MC difficulty, no special buffs
- Hostile mob types: zombie, skeleton, spider, creeper
- Passive mobs: cow, pig, sheep, chicken, horse, rabbit
- Villages: common (every 3-4 islands)
- Trees: oak, birch
- Loot chest: basic tools, bread, seeds, leather, iron nuggets
- Wind: none
- Island density: high (close together, 30-80 blocks apart)
- This is the learning zone. Safe, resource-poor but enough to get started

## Ring 1 -- Early Game
- Biomes: + Taiga, Dark Forest, Swamp
- Islands: medium-large (100-600 blocks wide)
- Ores: coal, iron, copper, lapis (uncommon), gold (rare)
- Mobs: +10% HP, +10% damage vs base MC values
- New hostile mobs: + witch, cave spider, slime, drowned
- Passive mobs: + wolf, fox, bee, turtle
- Villages: uncommon (every 5-6 islands)
- Trees: + spruce, dark oak
- Loot chest: iron tools, gold nuggets, enchanted books (low level), bows
- Wind: occasional light gusts (0.5 blocks/sec push)
- Island density: medium (50-150 blocks apart)

## Ring 2 -- Mid Game
- Biomes: + Desert, Savanna, Badlands, Jungle
- Islands: all sizes (30-800 blocks wide)
- Ores: coal, iron, copper, lapis, gold (common), diamond (very rare, 1-2 per island)
- Mobs: +25% HP, +25% damage
- New hostile mobs: + enderman, phantom, pillager, guardian (water islands)
- Passive mobs: + cat, parrot, llama, panda, donkey
- Villages: rare (every 8-10 islands)
- Trees: + acacia, jungle
- Loot chest: gold tools/armor, diamonds (1-2), enchanted books (mid level), potions
- Wind: moderate gusts (1-2 blocks/sec), direction changes every 30-60 sec
- Island density: medium-low (80-200 blocks apart)
- Pillager outposts appear on some islands

## Ring 3 -- Late Game
- Biomes: + Snowy Plains, Mushroom, all remaining overworld biomes
- Islands: all sizes, some very small dangerous platforms (10-30 blocks)
- Ores: all ores common, diamond uncommon (3-8 per island)
- Mobs: +50% HP, +50% damage
- New hostile mobs: + vindicator, evoker, ravager (raids more common)
- Loot chest: diamond tools, enchanted gear, golden apples, totems of undying (rare)
- Wind: strong gusts (2-3 blocks/sec), can blow player off bridge
- Island density: low (100-300 blocks apart)
- Some islands have mini-dungeons with spawners

## Ring 5
- Ores: diamond common, emerald uncommon
- Mobs: +100% HP, +100% damage (double base MC)
- Mob special effects: some mobs apply debuffs on hit (see mob_buffs.md)
- Wind: strong and unpredictable, sudden direction changes
- Small dangerous islands become more common

## Ring 10
- Ores: diamond abundant, lapis/emerald common
- Some island surfaces are partially made of iron ore blocks
- Mobs: +200% HP, +200% damage
- All hostile mobs can spawn during day (sky darkened in this ring)
- Wind: constant strong wind with random surges
- New environmental hazard: void lightning (see hazards.md)

## Ring 20
- Island surfaces start containing gold ore blocks mixed with stone
- Occasional small veins of iron blocks (not ore, actual iron blocks)
- Mobs: +500% HP, +500% damage
- Mobs spawn in larger groups (2x normal group size)
- Skeleton arrows deal fire damage
- Zombies have Speed I effect
- Creeper explosion radius +50%

## Ring 50
- Islands partially composed of ore blocks (iron, gold, diamond ore everywhere)
- Rare veins of gold blocks (2-5 blocks)
- Mobs: +1000% HP, +1000% damage
- Mobs spawn with enchanted gear (iron armor, enchanted weapons)
- Every mob inflicts at least one debuff on hit
- Wind: extreme, requires sneaking on bridges to not get blown off
- Island density: very low (200-500 blocks apart)
- Some islands are single-block-wide pillars of ore

## Ring 100
- Islands are almost entirely ore blocks and precious blocks
- Gold blocks appear in small patches (5-10 blocks)
- Diamond blocks appear rarely (1-3 per island)
- Emerald blocks appear rarely (1-2 per island)
- Mobs: +2000% HP, +5000% damage (one-shot most players even in diamond armor)
- All mobs have Speed II + Strength II effects
- Hostile mobs spawn in swarms (5-10 per group)
- Environmental: constant void lightning, extreme wind
- Only reachable by very well-prepared players

## Ring 200
- Islands frequently contain diamond blocks, gold blocks, emerald blocks
- Some small islands are entirely made of diamond/gold/emerald blocks (jackpot islands)
- Mobs: instant kill damage regardless of armor
- Mobs have Resistance II (take much less damage)
- Mob density: extremely high, every island is swarming
- This is endgame content -- risk/reward is extreme
- Strategy: sneak in, grab blocks, get out or die

## Ring 500+
- Scaling continues infinitely
- Mob HP and damage scale linearly: ring * 50 HP, ring * 25 damage
- Precious block frequency increases logarithmically (diminishing returns)
- Eventually every island surface is solid precious blocks
- Mobs become effectively unkillable, pure resource-grab gameplay
- Wind can reach 10+ blocks/sec, nearly impossible to stand on bridges

## Scaling Formulas
```
mob_hp_multiplier = 1.0 + (ring * 0.2)
mob_damage_multiplier = 1.0 + (ring * 0.25)
mob_speed_bonus = min(ring * 0.02, 1.0)  // cap at +100% speed

ore_density_multiplier = 1.0 + (ring * 0.1)
diamond_chance_per_block = min(0.001 + ring * 0.0005, 0.3)  // cap at 30%
precious_block_chance = max(0, (ring - 50) * 0.002)  // starts at ring 50

wind_strength = min(ring * 0.3, 15.0)  // blocks/sec, cap at 15
wind_gust_frequency = min(ring * 0.5, 30.0)  // gusts per minute

mob_group_size = min(1 + floor(ring / 10), 20)
mob_spawn_rate_multiplier = 1.0 + (ring * 0.05)
island_spacing = 50 + ring * 2  // blocks between islands, increases
```

## Ring Transition
- No hard border between rings
- Gradual transition over ~100 blocks
- Mobs near ring border use interpolated stats
- Player sees ring number change on HUD when crossing boundary
- Notification: "Entering Ring N" with brief description ("Danger increases")

## Death in High Rings
- Player respawns at bed (if set) or world spawn (ring 0)
- All items drop at death location
- Items in void are lost forever
- Getting back to a high ring death location is extremely difficult
- This makes high-ring exploration a high-stakes expedition
- Bring only what you can afford to lose

## Strategic Implications
- Players build outpost chains through rings: bed + chest every few islands
- Bridges between rings are permanent infrastructure, valuable community resource
- High-ring resource runs are group activities: one player mines, others defend
- Ender chests critical for safely banking resources from deep runs
