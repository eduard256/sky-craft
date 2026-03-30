# Multiplayer Mechanics

## Server Setup
- Docker image: `skycraft/server:latest`
- Default port: 35565
- Auth: automatic via central auth API (apiskycraft.webaweba.com)
- Server admin: first player to join becomes operator (configurable)
- Config file: `server.toml` in container volume

## Server Configuration
```toml
[server]
port = 35565
max_players = 50
view_distance = 8          # chunks
motd = "Welcome to Sky Craft"
difficulty = "normal"      # peaceful, easy, normal, hard

[world]
seed = 0                   # 0 = random seed on first start
world_name = "world"
spawn_protection = 16      # blocks around spawn no non-op can modify

[gameplay]
pvp = true
keep_inventory = false
mob_griefing = true
fire_tick = true
natural_regeneration = true
players_sleeping_percentage = 50
```

## Player Interaction

### PvP
- Configurable (default: on)
- Full MC combat: weapons, arrows, splash potions
- Player kills: victim drops all items + XP
- No PvP in spawn protection area
- No team system (V1) -- all players are independent

### Griefing Protection
- Spawn protection: configurable radius around world spawn
- No formal claim system (V1)
- Operators can rollback (future)
- Natural solution: build far away, high ring = nobody else goes there

### Player Visibility
- Nametag visible above player head within 64 blocks
- Sneaking: nametag hidden, reduced visibility distance to 32 blocks
- Invisible (potion): nametag fully hidden, detection range 8 blocks (if wearing armor)

## Shared Infrastructure

### Community Bridges
- Bridges are permanent structures in the world
- Any player can build on or extend bridges
- Any player can also destroy bridge blocks (no protection system V1)
- Natural trust-based system: griefing bridges hurts everyone
- Main highway bridges tend to emerge on active servers

### Shared Islands
- No island ownership system
- First player to place bed "claims" island informally
- Chests are NOT locked (any player can access)
- Ender chests are personal (standard MC behavior)
- Players naturally spread out to avoid conflict

### Communication
- Text chat visible to all players
- /msg for private messages
- Death messages broadcast to all players
- Ring transition messages only shown to transitioning player
- Server-wide announcements: operator can use /say command

## Server Performance

### Chunk Loading
- Chunks loaded in view_distance radius around each player
- Spawn chunks always loaded (standard MC)
- Mostly empty chunks (void) have minimal performance cost
- Island chunks have normal MC-level complexity
- Target: 20 TPS with 50 players at view_distance 8

### Entity Management
- Mobs in unloaded chunks: frozen (not simulated)
- Mobs in loaded chunks: full AI simulation
- Many void chunks = many empty chunks = less entity load than MC
- Island isolation naturally limits mob interactions

### World Save
- Autosave every 5 minutes
- Only modified chunks saved (unmodified = regeneratable from seed)
- World data stored in chunk files on disk (MC-style region format adapted for cubic chunks)
- Player data: separate file per player (inventory, position, stats)
