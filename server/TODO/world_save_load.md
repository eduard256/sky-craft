# TODO: World Save/Load

## Status: Not implemented (world is in-memory only, lost on restart)

## What exists
- `config.rs`: world_dir setting (default: ./world_data)
- `world/chunk.rs`: ChunkStore (DashMap in memory)
- Island generation is deterministic from seed (unmodified chunks can be regenerated)

## What needs to be built

### Chunk serialization
- Only save modified chunks (player-placed/broken blocks)
- Track "dirty" flag per chunk: set when block changed by player
- Format: bincode-serialized ChunkSection per file
- File path: `{world_dir}/chunks/r.{rx}.{rz}/c.{cx}.{cy}.{cz}.bin`
- Region grouping: 16x16 chunk columns per region file (like MC regions)
- On load: if chunk file exists, load from disk; if not, generate from seed

### Autosave
- Every 5 min (6000 ticks): save all dirty chunks
- On server shutdown (SIGTERM/SIGINT): save all dirty chunks
- Save in background thread to not block game loop
- Track save progress, log completion

### Player data
- Save per player: `{world_dir}/players/{uuid}.json`
- Data: position, rotation, health, hunger, saturation, XP, inventory, game mode,
  spawn point, bed position, statistics, active effects
- Save on disconnect and on autosave
- Load on connect (if file exists, otherwise use defaults)

### World metadata
- `{world_dir}/level.json`: seed, world age, time of day, weather state,
  difficulty, gamerules, spawn position
- Saved on autosave and shutdown
- Loaded on startup

### Chunk unloading
- Chunks with no player within 2x view_distance: candidate for unload
- Unload after 5 min of no nearby player
- If dirty: save to disk before unloading
- If clean (unmodified): just drop from memory (regeneratable)
- Spawn chunks: always loaded (never unload)

### Backup
- Before autosave: optionally copy previous save (rotate 3 backups)
- Admin command: /save-all (force immediate save)

### Files to create
- `server/src/world/save.rs` -- chunk serialization/deserialization
- `server/src/world/region.rs` -- region file management
- `server/src/player_data.rs` -- player save/load

### Estimated: ~1000 lines
