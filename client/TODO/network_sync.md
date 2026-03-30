# TODO: Network Sync & Prediction

## Status: Partial (connect + login works, no ongoing sync)

## What exists
- `network.rs`: connect(), send(), try_recv(), poll() implemented
- Login flow works (send token, receive LoginSuccess)
- `world.rs`: handle_server_packet() processes all server packet types
- `state.rs`: update() polls network in Playing state

## What needs to be built

### Continuous polling
- Run network.poll() on background tokio task
- Feed received packets into channel (mpsc)
- Main thread reads channel each frame (non-blocking)
- Currently network is async but main loop is sync (winit) -- need bridge

### Async-sync bridge
- Option A: tokio runtime on separate thread, communicate via channels
- Option B: poll network in update() with small timeout
- Option A preferred: separate thread for all network I/O
- Use std::sync::mpsc or crossbeam channel for packet passing:
  - network_thread -> main_thread: ServerPacket
  - main_thread -> network_thread: ClientPacket

### Client-side prediction
- Player movement: apply input locally, don't wait for server
- Send position to server, server validates
- If server corrects (PlayerPositionAndLook): snap to server position
- Smooth correction: interpolate toward server position over 3-5 frames
- Prediction buffer: store last N sent positions, compare with server acknowledgment

### Entity interpolation
- Server sends entity positions 20 times/sec
- Client renders at 60+ FPS
- Store last 2 known positions per entity with timestamps
- Interpolate between them based on elapsed time
- If packet delayed: extrapolate briefly, then freeze
- Smooth teleports: if distance > 8 blocks, teleport instantly; else interpolate

### Chunk request
- Track which chunks client expects to have loaded
- When player moves to new chunk column: expect new chunks from server
- If chunks don't arrive within timeout: request from server (future: explicit request packet)
- Currently server sends all chunks on login; dynamic streaming is server TODO

### Keep-alive
- Server sends KeepAlive every 15 sec
- Client must respond with KeepAliveResponse (same ID)
- Currently handled in server connection loop
- Client side: on receiving KeepAlive, immediately send response

### Disconnect handling
- Server sends Disconnect(reason): show reason on screen, return to main menu
- Connection drops (QUIC timeout): show "Connection lost", auto-reconnect option
- Clean up: clear world, entities, reset state

### Packet rate limiting
- Don't send position updates more than 20/sec (1 per tick)
- Batch multiple actions into single tick's packet burst
- Track last sent time, throttle if too fast

### Session persistence
- Save auth token to ~/.skycraft/session.json after successful login
- On startup: load token, skip login screen if valid
- On 401 from server: delete token, show login screen

### Files to create/modify
- `client/src/network/mod.rs` -- split into submodules
- `client/src/network/bridge.rs` -- async-sync channel bridge
- `client/src/network/prediction.rs` -- client-side movement prediction
- `client/src/network/interpolation.rs` -- entity interpolation
- `client/src/network/session.rs` -- token persistence

### Estimated: ~1500 lines
