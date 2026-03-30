# TODO: Lighting System

## Status: Not implemented (chunks have empty light arrays)

## What exists
- `ChunkSection` in protocol: block_light and sky_light Vec<u8> (currently empty)
- Generator places blocks but no light calculation

## What needs to be built

### Block light propagation
- Light sources: torch=14, lantern=15, glowstone=15, lava=15, etc
- BFS flood fill: start from each light source, spread to 6 neighbors
- Light decreases by 1 per block through air/transparent blocks
- Opaque blocks fully block light propagation
- Water: -3 per block horizontally
- When block placed/removed: recalculate affected area

### Sky light propagation
- Top of world: sky light = 15
- Propagate downward through transparent blocks
- First opaque block: sky light = 0 below it
- Horizontal spread: sky light spreads sideways -1 per block
- Night: sky light base drops to 4
- Rain/thunder: further reduction

### Light update algorithm
```
On block change at (x, y, z):
  1. Remove old light contribution (if light source removed)
  2. Add new light contribution (if light source placed)
  3. BFS flood from changed position
  4. Cap at chunk boundaries, mark neighbor chunks dirty
  5. Send updated light data to clients
```

### Performance
- Light updates can cascade through large areas
- Batch updates: collect all block changes per tick, process light once
- Async: run light calculation on separate thread
- Only send light data for chunks that changed

### Smooth lighting (client-side, not server)
- Server just sends per-block light levels
- Client interpolates for smooth rendering (ambient occlusion)

### Files to create
- `server/src/block/lighting.rs` -- light propagation engine

### Estimated: ~1000 lines
