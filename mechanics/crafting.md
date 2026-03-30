# Crafting & Smelting

## Crafting Grid
- Player inventory: 2x2 grid (simple recipes only)
- Crafting table: 3x3 grid (all recipes)
- Recipes are shaped (pattern matters) or shapeless (any arrangement)
- Output slot: click to take, shift-click crafts max possible

## Key Recipes (abbreviated, full list in data files)
- Planks: 1 log = 4 planks
- Sticks: 2 planks vertical = 4 sticks
- Crafting table: 4 planks in 2x2
- Tools: sticks + material in standard patterns (pick: 3 top + 2 sticks down)
- Furnace: 8 cobblestone ring
- Chest: 8 planks ring
- Bed: 3 wool top + 3 planks bottom
- Door: 6 planks in 2x3
- Ladder: 7 sticks in H pattern = 3 ladders
- Fence: 4 planks + 2 sticks = 3 fences
- Boat: 5 planks U shape
- Bucket: 3 iron ingots V shape
- Rail: 6 iron + 1 stick = 16 rails
- Bow: 3 sticks + 3 string
- Arrow: flint + stick + feather = 4 arrows
- Shield: 6 planks + 1 iron ingot
- Book: 3 paper + 1 leather

## Furnace / Smelting
- Input slot + fuel slot -> output slot
- Smelting time: 10 sec per item (200 ticks)
- Fuel values (items smelted per fuel):
  - Wood/planks: 1.5
  - Stick: 0.5
  - Coal/charcoal: 8
  - Coal block: 80
  - Lava bucket: 100
  - Blaze rod: 12
- Smelting recipes:
  - Raw iron -> iron ingot
  - Raw gold -> gold ingot
  - Raw food -> cooked food
  - Sand -> glass
  - Cobblestone -> stone
  - Clay ball -> brick
  - Log -> charcoal
  - Cactus -> green dye
  - Iron/gold tools/armor -> nugget (1)
- XP given when taking output: 0.1-1.0 per item depending on recipe

## Blast Furnace
- Smelts ores/metal armor 2x faster (5 sec)
- Only ores and metal items, not food
- Recipe: 3 iron + 1 furnace + 3 smooth stone

## Smoker
- Cooks food 2x faster (5 sec)
- Only food items
- Recipe: 1 furnace + 4 logs

## Anvil
- Combines/renames items, applies enchanted books
- Costs XP levels, max cost 39 levels
- Damaged by use, 3 stages: anvil -> chipped -> damaged -> breaks
- Recipe: 3 iron blocks + 4 iron ingots

## Enchanting Table
- Enchants tools/armor/weapons using XP + lapis lazuli
- 3 enchantment options shown, costs 1-3 lapis + 1-3 levels
- Bookshelves within 1 block boost max level (15 bookshelves = max level 30)
- Enchantments: Protection, Sharpness, Efficiency, Fortune, Silk Touch,
  Unbreaking, Mending, Looting, Power, Infinity, etc.

## Grindstone
- Removes enchantments, returns some XP
- Combines 2 same items = durability sum + 5% bonus
- Recipe: 2 sticks + 1 stone slab + 2 planks

## Stonecutter
- Cuts stone variants more efficiently (1:1 vs crafting ratios)
- Recipe: 3 stone + 1 iron ingot

## Brewing Stand
- Brews potions: water bottle + ingredient -> potion
- Fuel: blaze powder (20 brews per)
- Base: awkward potion (nether wart + water bottle)
- Effects: healing, regen, strength, speed, fire resist, night vision, etc.
- Modifiers: redstone = longer duration, glowstone = stronger effect
- Gunpowder = splash potion, dragon breath = lingering potion (skip dragon breath)
