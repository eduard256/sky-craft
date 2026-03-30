# TODO: Entity Rendering

## Status: Not implemented

## What exists
- `world.rs`: ClientEntity struct with position, rotation, type, custom_name
- Entities tracked from server packets (spawn, move, destroy)

## What needs to be built

### Entity models
- Each entity type has a model (segmented body parts)
- Model definition: list of cuboid parts (body, head, legs, arms, etc)
- Each part: position, size, rotation pivot, texture UV region
- Player model: Steve (4px wide arms) or Alex (3px wide arms)
- Mob models: simplified versions of MC models
  - Zombie/skeleton/player: humanoid (head, body, 2 arms, 2 legs)
  - Cow/pig/sheep: quadruped (head, body, 4 legs)
  - Chicken: small biped (head, body, 2 legs, 2 wings)
  - Spider: wide body + 8 legs
  - Creeper: body + 4 legs, no arms
  - Slime: single cube, size varies
- Start with simple colored cubes per entity type for v0.0.1

### Entity model loading
- Define models in code or load from JSON files
- Each model part: vertex data for a textured cuboid
- Texture: entity skin texture from client/assets/textures/minecraft/textures/entity/
- UV mapping: each face of each cuboid maps to region of entity texture

### Entity rendering pipeline
- Separate from block rendering (different vertex format, different shader)
- Vertex: position + tex_coord + normal
- Uniform per entity: model matrix (position + rotation + scale)
- Draw each entity model part as separate mesh or batched

### Animation
- Idle: slight body bob, arm swing
- Walking: leg alternating swing, arm counter-swing
- Attack: arm swing forward
- Hurt: red flash overlay (0.5 sec)
- Death: entity falls sideways and fades
- Animation = per-part rotation over time (sin wave for walk cycle)
- Interpolate between server position updates (3-5 tick delay)

### Entity interpolation
- Server sends position 20 times/sec
- Client renders at 60+ FPS
- Interpolate between last 2 known positions
- Smooth movement, hide network jitter
- Extrapolate slightly if packet delayed

### Nametags
- Render player names above entity heads
- Billboard quad (always faces camera)
- Text rendered to texture or draw with glyph system
- Visible within 64 blocks, hidden when sneaking (32 blocks)
- Show hearts/HP bar for mobs being looked at (optional)

### Shadow
- Simple circular shadow blob below each entity
- Project onto ground surface
- Fade with distance from ground
- Darken ground texture in shadow area

### Item entities
- Dropped items on ground: render as small 3D block or 2D sprite
- Bob up and down slowly (sin wave)
- Rotate slowly around Y axis
- Merge visual for stacked items

### Particles
- Block break particles: small cubes with block texture, scatter on break
- Hit particles: red slash marks
- Critical hit: star particles
- Potion: swirl particles around entity
- Fire: flame particles on burning entities
- XP orbs: green glowing spheres that fly toward player
- Environmental: dripping water, lava sparks, void particles

### Files to create
- `client/src/renderer/entity.rs` -- entity mesh, animation, rendering
- `client/src/renderer/model.rs` -- entity model definitions
- `client/src/renderer/particle.rs` -- particle system
- `client/src/renderer/nametag.rs` -- billboard text rendering

### Estimated: ~3000 lines
