# Rendering (Client)

## Voxel Rendering
- Each block = 16x16x16 pixel texture, 6 faces
- Only render exposed faces (face culling): skip faces between 2 solid blocks
- Greedy meshing: merge adjacent same-texture faces into larger quads
- Chunk mesh: each 16x16x16 chunk = 1 mesh, rebuilt when blocks change
- Transparent blocks (glass, water, leaves): separate render pass, alpha blending
- LOD: distant chunks rendered with simplified mesh (future optimization)

## Textures
- MC texture atlas: all block/item textures in single atlas (16x16 or 32x32 per tile)
- Texture coordinates per face, UV mapping
- Animated textures: water, lava, fire, portal = multi-frame strip, cycle at fixed rate
- Block models: JSON-defined (MC format), most are cubes, some complex (stairs, fences, etc)
- Item models: flat sprite or 3D model

## Entity Rendering
- Mob models: segmented body parts (head, body, arms, legs), animated with keyframes
- Player model: Steve/Alex base, 4px skin overlay layer
- Item in hand: rendered at hand bone position
- Shadow: circular shadow blob projected on ground below entity
- Name tags: rendered above entity head, billboard (always faces camera)
- Particles: quad billboards, various types (block break, crit, hearts, smoke, etc)

## Sky & Atmosphere
- Sky color: gradient based on time of day and biome
- Sun/moon: textured quads on sky sphere, rotate with time
- Stars: visible at night, small dots
- Clouds: flat layer at Y=192 (or adapted height), drift slowly
- Fog: distance-based, color matches sky horizon
- Sky Craft special: void below islands = dark gradient fading to black/blue
- Void particles: subtle floating particles in the void for depth feel

## Lighting Render
- Per-vertex or per-face lighting from server light data
- Ambient occlusion: darken corners/edges of blocks
- Smooth lighting: interpolate light values across face
- Dynamic light from held torch (client-side only, no server involvement)

## Water Rendering
- Translucent blue tint, animated texture
- Water surface: slight wave animation (vertex displacement or texture scroll)
- Underwater: blue fog, distortion effect
- Reflections: optional, basic sky color reflection on surface

## UI / HUD
- Hotbar: bottom center, 9 slots with item icons
- Health bar: 10 hearts above hotbar
- Hunger bar: 10 shanks, right side
- XP bar: green bar below hotbar, level number
- Armor bar: above hearts when wearing armor
- Air bubbles: appear underwater, above hunger bar
- Crosshair: center screen
- Chat: bottom-left, messages fade after 10 sec
- Debug screen (F3): coords, FPS, chunk info, light level, biome
- Inventory screen: player model, grid, crafting
- Container screens: chest, furnace, enchanting table, etc

## Performance Targets
- 60+ FPS at 8 chunk render distance on mid-range GPU
- 144+ FPS on high-end
- Chunk mesh generation: async, background thread pool
- Frustum culling: only render chunks in camera view
- Occlusion culling: skip chunks fully hidden behind other chunks
- Draw distance: configurable 2-32 chunks
