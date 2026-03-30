# Networking (Client-Server Protocol)

## Architecture
- Authoritative server: all game state on server, client is dumb renderer
- Client sends: input (movement, actions, block placement, chat)
- Server sends: world state, entity positions, block changes, events
- Binary protocol over TCP (custom, inspired by MC protocol structure)
- Connection: TCP socket to server ip:port (default 25565)

## Connection Flow
1. Client connects TCP
2. Client sends Handshake packet (protocol version)
3. Client sends Login packet (auth token from session.json)
4. Server validates token via auth API (POST /auth/validate-token)
5. Server sends Login Success (player UUID, nickname) or Disconnect (reason)
6. Server sends initial data: spawn position, world time, difficulty
7. Server starts streaming chunk data around player

## Packet Structure
- Length (varint) + Packet ID (varint) + Data (bytes)
- Compression: optional zlib compression for packets > 256 bytes
- No encryption in V1 (auth token is the security layer)

## Key Packet Types

### Client -> Server
- Player Position (x, y, z, on_ground) -- sent every tick when moving
- Player Look (yaw, pitch) -- sent when camera moves
- Player Action (start/stop mining, use item, swap hands)
- Block Placement (position, face, hand)
- Block Break (position)
- Chat Message (text)
- Keep Alive response (echo server's keep alive ID)
- Entity Interact (entity ID, action: attack/interact)
- Click Window (inventory slot interactions)
- Held Item Change (hotbar slot 0-8)

### Server -> Client
- Chunk Data (chunk coords + block data, compressed)
- Block Change (position + new block state)
- Entity Spawn (entity ID, type, position, metadata)
- Entity Move (entity ID, delta position, rotation)
- Entity Destroy (entity ID list)
- Player Info (add/remove players, update ping)
- Set Health (HP, hunger, saturation)
- Set Experience (XP bar, level, total XP)
- Update Time (world age, time of day)
- Chat Message (sender, text, type)
- Keep Alive (random ID, client must respond)
- Disconnect (reason text)
- Sound Effect (sound ID, position, volume, pitch)
- Particle (type, position, count, data)
- Block Entity Data (sign text, chest contents, etc)
- Window Items (inventory/container contents)

## Chunk Loading
- Server sends chunks in render distance around player (default 8 chunks)
- When player moves into new chunk: server sends new edge chunks, client unloads far ones
- Chunk data: array of block states (palette-compressed like MC)
- Light data sent alongside chunk data

## Tick Sync
- Server runs at 20 TPS (50ms per tick)
- Client renders at unlimited FPS, interpolates entity positions between server updates
- Client-side prediction for player movement (predict locally, correct from server)
- Entity interpolation: smooth movement over 3-5 ticks of delay

## Keep Alive
- Server sends every 15 sec
- Client must respond within 30 sec or gets disconnected
- Measures ping (round trip time)
