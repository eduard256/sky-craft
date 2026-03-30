# Lighting

## Light Levels (0-15)
- 0 = total darkness, hostile mobs spawn
- 15 = max brightness (sunlight, glowstone, lava, lantern)
- Light decreases by 1 per block of distance from source

## Sky Light
- Sun provides level 15 from above during day
- Reduces through transparent blocks (leaves, water = -1 per block)
- At night: sky light drops to 4 (moonlight)
- Rain/thunder: further reduces sky light
- No sky light below solid blocks (shadow)

## Block Light Sources
- Torch: 14
- Lantern: 15
- Glowstone: 15
- Lava: 15
- Fire: 15
- Jack o'Lantern: 15
- Sea lantern: 15
- Redstone torch: 7
- Brewing stand: 1
- Furnace (active): 13
- Ender chest: 7
- Magma block: 3
- Campfire: 15
- Soul campfire: 10

## Light Propagation
- Calculated per block in all 6 directions
- Transparent blocks (glass, leaves): -1 per block
- Opaque blocks: fully block light
- Water: -3 per block horizontally
- Light updates propagate when blocks placed/removed (BFS flood fill)
- Server calculates, sends to client per chunk

## Smooth Lighting (Client-Side)
- Ambient occlusion: darken block faces in corners/edges
- Interpolate light between adjacent blocks for smooth gradients
- Pure client rendering, server sends raw light values
