# Inventory & Containers

## Player Inventory
- 36 slots: 9 hotbar + 27 main inventory
- 4 armor slots: helmet, chestplate, leggings, boots
- 1 offhand slot (shield, torch, map, etc)
- 2x2 crafting grid
- Most items stack to 64, some to 16 (ender pearls, eggs, snowballs), some to 1 (tools, armor)

## Hotbar
- 9 slots, selected with 1-9 keys or scroll wheel
- Selected item shown in main hand, used with left/right click
- Swap items with number keys, F key swaps with offhand

## Chest
- 27 slots (3 rows of 9)
- Double chest: 2 chests adjacent = 54 slots
- Opens with right-click, can't open if solid block on top
- Drops all contents when broken
- Can be locked with name tag in adventure mode (not relevant for us)
- Trapped chest: emits redstone signal based on viewers (skip for now)

## Ender Chest
- 27 slots, personal per player
- All ender chests share same inventory for that player
- Drops 8 obsidian when broken (silk touch = ender chest)
- Good for Sky Craft: safe storage across islands

## Barrel
- 27 slots like chest, but opens even with block on top
- Functionally identical to chest otherwise

## Shulker Box
- 27 slots, keeps items when broken (portable storage)
- Can be placed and picked up repeatedly
- Can be dyed any color
- NOTE: normally from End. For Sky Craft: could be crafted differently or found in loot chests

## Hopper (redstone postponed but still a container)
- 5 slots, transfers items between containers
- 1 item per 0.4 sec (8 ticks)
- Pulls from container above, pushes to container below/facing
- Disabled by redstone signal -- skip for V1

## Dropper / Dispenser (redstone, skip V1)

## Item Frames
- Display 1 item on wall, decorative
- Rotates item when right-clicked (8 positions)
- Drops item + frame when broken
