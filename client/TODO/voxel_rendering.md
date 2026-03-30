# TODO: Voxel Rendering

## Status: Basic rendering DONE, optimizations and textures remaining

## What is DONE
- [x] Chunk meshing with face culling (solid/air neighbor check)
- [x] Vertex format: position + color + normal (bytemuck Pod)
- [x] WGSL shader with directional lighting
- [x] Render pipeline: vertex/fragment shaders, depth buffer, backface culling
- [x] Camera: first-person, WASD + mouse look, perspective projection, FOV 70
- [x] Chunk mesh management: per-chunk GPU vertex/index buffers, dirty flag
- [x] Draw calls: 1 per chunk, camera uniform bound once
- [x] Demo world for offline testing

## What still needs to be built

### Greedy meshing
- Merge adjacent same-color faces into larger quads
- Dramatically reduces triangle count (10-50x fewer triangles)
- Process one face direction at a time
- Scan for rectangular regions of same block type
- Current: 1 quad per block face. Target: 1 quad per contiguous region

### Texture atlas (replaces current color-based rendering)
- Load all block textures from client/assets/textures/minecraft/textures/block/
- Pack into single large texture atlas (2048x2048)
- Map block_state_id -> atlas UV coordinates
- Update vertex format: add tex_coords (f32x2) replacing color
- Update shader: sample texture atlas instead of using vertex color
- Parse blocks.json for block_state -> texture name mapping

### Block state -> texture mapping
- Multi-face blocks: top/bottom/side different (grass_block, log, furnace)
- Directional blocks: rotated based on state (stairs, logs with axis)
- Animated textures: water, lava, fire (multi-frame strip, cycle)

### Frustum culling
- Test chunk AABB against camera frustum before drawing
- 6-plane frustum extraction from view-projection matrix
- Skip chunks outside view (50-70% reduction in draw calls)

### Transparent blocks
- Glass, water, leaves, ice: separate render pass with alpha blending
- Sort transparent chunks back-to-front
- Water: animated UV offset for flowing appearance

### Render distance + fog
- Only mesh chunks within view distance
- Fog at render distance edge blending to sky color
- Configurable: 2-32 chunks

### Background mesh generation
- Move chunk meshing to background thread (currently blocks main thread)
- Mesh queue: prioritize chunks closest to player
- On block change: regenerate affected chunk + neighbors

### Files to create
- `client/src/renderer/atlas.rs` -- texture atlas generation

### Estimated remaining: ~2500 lines
