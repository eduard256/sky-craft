# TODO: Settings & Persistence

## Status: Not implemented

## What exists
- No settings file, no persistence
- `audio.rs`: volume fields exist but not saved

## What needs to be built

### Settings file
- Path: ~/.skycraft/settings.json
- Load on startup, save on change
- Categories:
  - Video: render_distance, fov, max_fps, vsync, clouds, particles, fancy_graphics
  - Audio: master_volume, music_volume, sfx_volume, ambient_volume, void_ambience_volume
  - Controls: sensitivity, invert_y, key_bindings (map action -> key)
  - Game: chat_visible, hud_scale, language (English only v0.0.1)

### Key rebinding
- Default bindings defined in code
- Override from settings file
- Settings screen: click binding -> press new key -> save
- Conflict detection: warn if key already bound

### Session persistence
- Path: ~/.skycraft/session.json
- Store: { nickname, token, last_server_address, last_server_port }
- Load on startup: auto-fill connect screen
- Save after successful login
- Delete on auth failure (401)

### Server history
- Path: ~/.skycraft/servers.json
- Store list of: { name, address, port, last_connected_time }
- Show in connect screen as "Recent Servers"
- Add entry on successful connect
- Max 20 entries, drop oldest

### Screenshots
- F2 key: capture current frame
- Save to ~/.skycraft/screenshots/YYYY-MM-DD_HH-MM-SS.png
- Chat notification: "Screenshot saved"

### Files to create
- `client/src/settings.rs` -- settings load/save, defaults
- `client/src/session.rs` -- auth token persistence
- `client/src/servers.rs` -- server history

### Estimated: ~500 lines
