# TODO: Inventory & Crafting System

## Status: Not implemented (stubs only)

## What exists
- `player.rs`: inventory Vec<Slot> with 46 slots, held_slot, cursor_item
- `game.rs`: handle_click_slot() stub, handle_block_place() consumes 1 item
- Protocol: ClickSlot, WindowItems, SetSlot, OpenWindow packets defined

## What needs to be built

### Inventory click handling
- Normal click: pick up / place item at slot
- Shift-click: move item to other section (hotbar <-> main, inventory -> container)
- Number key (1-9): swap slot with hotbar
- Drop (Q): drop 1 item, Ctrl+Q drop whole stack
- Double-click: collect all same items into cursor stack
- Drag: paint items across slots (left=split evenly, right=1 each)
- Middle-click: creative mode copy stack
- Server must validate every click and send correction if invalid

### Crafting (2x2 and 3x3)
- Load recipes from `common/data/recipes.json` at startup
- Parse recipe format: shaped (pattern + key) and shapeless (ingredients list)
- On every inventory change in crafting grid: check all recipes for match
- If match: show result in output slot
- On take from output: consume ingredients, check if another craft possible
- 2x2 grid: slots 41-44 in player inventory, output slot 45
- 3x3 grid: opened via crafting table block, separate window (id != 0)

### Recipe matching algorithm
```
For shaped recipe:
  For each rotation (0, 90, 180, 270) and flip:
    Check if pattern matches grid contents
    Match = all pattern slots have correct item type

For shapeless recipe:
  Check if grid contains exactly the required ingredients (any arrangement)
```

### Container windows
Each container type needs open/close/click handling:
- Chest (27 slots), Double Chest (54 slots)
- Furnace (3 slots: input, fuel, output) -- needs smelting tick logic
- Blast Furnace (same as furnace, 2x speed, ores only)
- Smoker (same as furnace, 2x speed, food only)
- Crafting Table (9+1 crafting grid)
- Enchanting Table (2 slots: item + lapis, 3 options display)
- Anvil (3 slots: input1 + input2 + output, XP cost)
- Brewing Stand (5 slots: 3 bottles + ingredient + fuel)
- Grindstone (3 slots: 2 input + output, removes enchants)
- Stonecutter (2 slots: input + output, recipe selection)
- Barrel, Shulker Box (27 slots each, like chest)
- Loom, Cartography Table, Smithing Table (unique UIs)
- Beacon (1 slot, effect selection)

### Furnace smelting logic
- Load smelting recipes from recipes.json
- Tick every 10 sec (200 ticks) to smelt 1 item
- Track fuel burn time remaining
- Track smelt progress (0-200 ticks)
- Output XP when taking smelted items
- Blast furnace: 5 sec per item
- Smoker: 5 sec per item

### Item stack rules
- Max stack size per item type (mostly 64, some 16, tools/armor = 1)
- Stack merging when picking up / shift-clicking
- Durability tracking for tools/armor
- Enchantment data preservation

### Files to create
- `server/src/inventory/mod.rs` -- click handler, slot validation
- `server/src/inventory/crafting.rs` -- recipe matching
- `server/src/inventory/furnace.rs` -- smelting tick logic
- `server/src/inventory/container.rs` -- container open/close/window management

### Estimated: ~3000 lines
