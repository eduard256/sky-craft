# Player Movement & Physics

## Walking / Running
- Walk speed: 4.317 blocks/sec
- Sprint speed: 5.612 blocks/sec (30% faster)
- Sneak speed: 1.295 blocks/sec, prevents falling off edges
- Sprint: double-tap W or hold Ctrl+W
- Sneak: hold Shift, prevents block edge fall, shows name through walls shorter distance

## Jumping
- Jump height: 1.25 blocks (clears 1 block gap)
- Sprint-jump: ~3.6 blocks forward distance
- Jump boost potion: +0.5 blocks per level
- Auto-jump: optional, automatically jumps when walking into 1-block step

## Swimming
- Surface swim: hold jump in water, ~2.2 blocks/sec
- Sprint swim: double-tap W in water = dolphin-kick style, ~5.6 blocks/sec
- Sinking: stop moving = slowly sink
- Depth Strider boots: faster underwater movement (+1 block/sec per level, max III)

## Falling
- Gravity: 32 blocks/sec^2 acceleration
- Terminal velocity: ~78 blocks/sec (after ~3.5 sec of falling)
- Fall damage: (fall_distance - 3) HP, so 4+ blocks = damage
- Feather Falling boots: reduces fall dmg by 12% per level (max IV = 48%)
- Water/slime block/hay bale: negates fall damage
- Slow Falling potion: float down slowly, no fall damage

## Climbing
- Ladders: press against ladder block to climb, ~2.35 blocks/sec up
- Vines: same as ladders if block behind vine
- Scaffolding: climb up/down, sneak to stay at level

## Boat
- 8 blocks/sec on water, 2 blocks/sec on land (slow)
- 2 passenger seats, no fuel needed
- Breaks if hits land too fast, drops boat item
- Controlled with WASD, mouse to look

## Minecart
- On rails: max ~8 blocks/sec on powered rails
- Powered rail: boosts speed with redstone signal
- Various types: chest minecart, hopper minecart, TNT minecart
- NOTE: postpone until redstone V2

## Elytra (maybe future)
- Glide from high points, rocket boost for flight
- Durability depletes during flight, repaired with phantom membrane
- Could be great for Sky Craft island traversal (future feature)
