# TODO: Voxel Rendering

## Status: Not implemented (screen clears to sky color only)

## What exists
- `renderer.rs`: wgpu device/surface/queue init, render pass with clear color
- `world.rs`: ClientWorld stores chunks with block data, get_block() works

## What needs to be built

### Chunk meshing
- For each loaded chunk section (16x16x16): generate triangle mesh
- Per-block: check 6 faces, only emit face if adjacent block is air/transparent
- Face culling: skip faces between two opaque blocks (biggest optimization)
- Each face = 2 triangles = 6 vertices (or 4 verts + 6 indices with index buffer)
- Vertex format: position (f32x3) + tex_coords (f32x2) + normal (f32x3) + light (f32)
- Use bytemuck for GPU-compatible vertex struct

### Greedy meshing
- Merge adjacent same-texture faces into larger quads
- Dramatically reduces triangle count (10-50x fewer triangles)
- Process one face direction at a time (e.g. all +Y faces in a chunk)
- Scan for rectangular regions of same block type
- Emit one large quad per region instead of per-block face

### Texture atlas
- Load all block textures from client/assets/textures/minecraft/textures/block/
- Pack into single large texture atlas (e.g. 1024x1024 or 2048x2048)
- Each block texture = 16x16 pixels, atlas has grid of tiles
- Map block_state_id -> atlas UV coordinates
- Need mapping table: block_state -> texture file name -> atlas position
- Parse from blocks.json + file system scan

### Block state -> texture mapping
- Load common/data/blocks.json for block state IDs and names
- Map block name to texture filename(s):
  - Simple blocks: 1 texture all faces (stone, dirt, sand)
  - Multi-face: top/bottom/side different (grass_block, log, furnace)
  - Directional: rotated based on block state (stairs, logs with axis)
- Start with simplified mapping: name -> single texture for v0.0.1

### Render pipeline
- Vertex shader: transform position by view-projection matrix
- Fragment shader: sample texture atlas at UV, apply light level
- Depth buffer: standard Z-buffer for correct occlusion
- Backface culling: enabled (never see inside of blocks)
- Bind groups: texture atlas (group 0), camera uniform (group 1)

### Camera
- First-person camera at player eye height (1.62 blocks above feet)
- Projection: perspective, FOV 70 degrees, near=0.1, far=1000
- View matrix: from player position + yaw/pitch rotation
- Mouse look: yaw (horizontal) and pitch (vertical) from mouse delta
- Pitch clamped to -89..+89 degrees

### Chunk mesh management
- Generate mesh on background thread when chunk data arrives
- Store GPU vertex/index buffers per chunk
- On chunk unload: drop GPU buffers
- On block change: regenerate affected chunk mesh + neighbors (if face visibility changed)
- Mesh generation queue: prioritize chunks near player

### Draw call batching
- Each chunk = 1 draw call (1 vertex buffer + 1 index buffer)
- Set camera uniform once, bind texture atlas once
- Loop through visible chunks, draw each
- Sort chunks front-to-back for early Z rejection

### Frustum culling
- Before drawing chunk: test chunk AABB against camera frustum
- Skip chunks fully outside view frustum
- 6-plane frustum test against chunk bounding box
- Reduces draw calls by 50-70% typically

### Transparent blocks
- Glass, water, leaves, ice: separate render pass
- Sort transparent chunks back-to-front (painter's algorithm)
- Enable alpha blending for transparent pass
- Water: separate mesh layer with animated UV offset

### Render distance
- Only mesh and draw chunks within player's view distance
- Configurable: 2-32 chunks
- Default: 8 chunks
- Fog at render distance edge (blend to sky color)

### Files to create
- `client/src/renderer/mod.rs` -- pipeline setup, frame rendering
- `client/src/renderer/mesh.rs` -- chunk meshing + greedy meshing
- `client/src/renderer/atlas.rs` -- texture atlas generation
- `client/src/renderer/camera.rs` -- camera matrices, frustum
- `client/src/renderer/shader.wgsl` -- vertex + fragment shaders
- `client/src/renderer/vertex.rs` -- vertex struct definition

### Estimated: ~4000 lines
