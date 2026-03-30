# Bridge Building & Island Traversal

## Block Placement Rules
- Identical to MC: block can only be placed against an existing solid block face
- Cannot place blocks floating in air with no neighbor
- Once placed, blocks stay even if supporting neighbor is removed (no structural collapse)
- Sand/gravel obey gravity: fall if no block below, into void = destroyed
- Sneaking (Shift) prevents falling off edges while building

## Bridge Building Strategy

### Basic Bridge (Ring 0-2)
- 1-block-wide cobblestone path
- Sneak to edge, look down at side of last block, place next block
- Cheap materials, easy to build
- Dangerous in wind (ring 1+): player can be pushed off
- No railing needed at low rings

### Reinforced Bridge (Ring 3-10)
- 3-block-wide path with fence/wall railings
- Railings prevent wind from pushing player off (wind blocked by solid blocks)
- More material-intensive but much safer
- Recommended width: 3 blocks path + 1 fence each side = 5 total

### Enclosed Bridge (Ring 10+)
- Full tunnel: floor + walls + ceiling
- Blocks wind completely, prevents void lightning strikes
- Expensive (many blocks per meter) but safest
- Can light interior with torches (prevents mob spawning inside)
- Glass ceiling/walls for visibility while staying protected

### Emergency Bridge
- Sprint-jump bridging: faster but risky, experienced players only
- Water bucket trick: place water at destination to break fall if you miss
- Ender pearl: throw to reach distant island (8-40 block range, unreliable in wind)

## Bridge Hazards

### Mob Spawning on Bridges
- Dark bridges (no torches) in high rings: mobs spawn on bridge surface
- Light your bridges every 8 blocks (MC light falloff rules)
- Mob on 1-wide bridge = nearly impossible to pass without fighting or falling
- Spiders can climb bridge walls and drop on player from above/below

### Bridge Decay (Ring 15+)
- Unvisited bridge segments (no player within 5 chunks for 7+ real days):
  random blocks can disappear (1-2 per day)
- Simulates weathering/void erosion
- Players must maintain bridges by visiting or using more durable blocks
- Obsidian bridges never decay
- Stone/cobble: slow decay (1 block per 7 days)
- Wood: fast decay (2-3 blocks per 7 days)
- Dirt/sand: very fast decay (5+ blocks per 7 days)

### Wind + Bridge Width
```
1-block bridge: full wind effect, very dangerous above ring 3
2-block bridge: 80% wind effect
3-block bridge: 50% wind effect
4+ block bridge: 30% wind effect
Enclosed bridge (walls+ceiling): 0% wind effect
```

## Alternative Traversal

### Ender Pearls
- Throwable, teleport player to landing location
- Range: 30-40 blocks in still air
- Wind affects pearl trajectory at high rings
- 5 damage on landing (2.5 hearts)
- Risky in void: miss = death
- Obtainable from endermen (ring 2+) and cleric villager trades

### Boat Launcher
- Place water source -> place boat -> ride boat off edge
- Boat falls with player, takes fall damage
- If aimed at water on target island: safe landing
- If aimed at ground: boat breaks, player takes fall damage
- Risky but faster than building bridge for nearby islands

### TNT Cannon (Advanced)
- Build TNT cannon to launch player via explosion boost
- Extremely high skill ceiling
- Can cross 50-100 block gaps in one shot
- Dangerous: miscalculation = void death
- Requires redstone (post-V1 feature)

### Slime Block Launch Pad (Post-V1)
- Piston + slime block launcher
- Bounces player high and far
- Requires redstone mechanics

### Scaffolding Tower
- Build scaffolding tower up, then bridge across at higher Y
- When done, break bottom scaffolding block = entire column collapses
- Useful when target island is higher than source
- Scaffolding is cheap (bamboo + string)

## Navigation Aids

### Compass Behavior
- Standard MC compass points to world spawn (ring 0 center)
- Useful for finding direction home
- At high rings: compass helps orient toward center

### Maps
- Standard MC map crafting works
- Maps show island outlines and bridges
- Extremely useful for navigating island archipelagos
- Map wall at base: shows explored territory

### Coordinates
- F3 debug screen shows XYZ coordinates
- Ring number calculated from XZ distance to origin
- Players share coordinates to meet up or mark interesting islands

### Waypoint System
- Place a sign with specific text format: "[Waypoint] Name"
- Appears on player's HUD as directional marker when within 200 blocks
- Maximum 10 active waypoints per player
- Waypoints visible through blocks (like MC nametags)
