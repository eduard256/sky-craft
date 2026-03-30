# Gravity & Block Physics

## Gravity Blocks
- Sand, gravel, red sand, concrete powder, anvil, dragon egg (n/a)
- Fall when no solid block below
- Fall speed: same as entities, accelerates with gravity
- Landing on non-solid block (torch, slab, sign): drops as item
- Landing on solid block: places as block
- Falling into void: destroyed, no drop

## Entity Physics
- All entities affected by gravity (32 blocks/sec^2)
- Drag in air: velocity * 0.98 per tick
- Drag in water: velocity * 0.8 per tick
- Knockback: applied on hit, modified by enchants/sprinting
- Entity collision: players push each other slightly, mobs have hitboxes
- Item entities: bounce on landing, despawn after 5 min, merge same items within 1 block

## Block Updates
- When block placed/removed: notify 6 adjacent blocks
- Adjacent blocks check if still valid (e.g., torch needs solid wall, crops need farmland)
- Invalid blocks break and drop as items
- Water/lava recalculate flow
- Sand/gravel check for support

## Piston Physics (postpone V1, redstone)
- Sticky piston: push + pull blocks
- Regular piston: push only
- Can push up to 12 blocks in a line
- Some blocks unmovable: obsidian, bedrock, enchanting table, ender chest
- Slime blocks: bounce entities, stick to adjacent blocks when pushed
