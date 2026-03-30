# TODO: Dynamic Chunk Streaming

## Status: Basic (sends all chunks on login, no dynamic loading)

## What exists
- `network/connection.rs`: send_initial_chunks() sends all chunks in view_distance on login
- No tracking of which chunks player has loaded
- No sending new chunks when player moves

## What needs to be built

### Per-player loaded chunk tracking
- Track set of chunk positions loaded on each client
- On player movement into new chunk column:
  - Calculate which new chunks to send (entered view range)
  - Calculate which old chunks to unload (left view range)
  - Send ChunkData for new chunks
  - Send UnloadChunk for old chunks

### View distance handling
- Player can change view_distance in ClientSettings
- On change: recalculate loaded set, send/unload diff
- Server-side max: config.view_distance
- Effective: min(player.view_distance, config.view_distance)

### Chunk send prioritization
- Prioritize chunks closest to player (send center first, edges later)
- Prioritize chunks in look direction
- Don't send all-air chunks (void) -- skip them
- Rate limit: max N chunks per tick per player to avoid network flood
- Queue system: pending_chunks per player, drain N per tick

### Chunk generation threading
- Move chunk generation off main thread
- Use thread pool or tokio::spawn_blocking for generation
- Cache generated chunks (they're already in ChunkStore)
- Generate chunk on first request, cache for subsequent players

### Entity visibility
- When chunk loads: send SpawnEntity for all entities in that chunk
- When chunk unloads: send DestroyEntities for entities leaving view
- Track which entities each player can see
- Only send entity movement/updates to players who have that entity visible

### Files to modify/create
- Modify `network/connection.rs`: replace blocking initial send with async streaming
- Create `server/src/network/chunk_stream.rs`: per-player chunk manager
- Modify `game.rs`: track player chunk movement, trigger chunk send/unload

### Estimated: ~800 lines
