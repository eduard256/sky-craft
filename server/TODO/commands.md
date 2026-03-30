# TODO: Full Command System

## Status: Basic commands only (/help, /list, /seed, /ring, /stats, /spawn)

## What exists
- `game.rs`: handle_command() with 6 basic commands
- No operator permission checking

## What needs to be built

### Permission system
- Operator list: stored in ops.json file
- First player to join: auto-op (configurable)
- /op and /deop commands to manage
- Non-op players: only /help, /msg, /list, /spawn, /ring, /stats

### Player commands (everyone)
- `/msg <player> <text>` -- private message (whisper)
- `/me <action>` -- action message (*player waves*)

### Operator commands
- `/gamemode <survival|creative|spectator> [player]` -- change game mode
- `/tp <player> <x> <y> <z>` -- teleport player to coords
- `/tp <player1> <player2>` -- teleport player1 to player2
- `/give <player> <item_id> [amount]` -- give items
- `/time set <day|night|noon|midnight|value>` -- set time
- `/time add <value>` -- add to time
- `/weather <clear|rain|thunder> [duration]` -- set weather
- `/kill [player|@e]` -- kill player or entities
- `/ban <player> [reason]` -- ban by nickname
- `/unban <player>` -- remove ban
- `/kick <player> [reason]` -- kick from server
- `/op <player>` -- grant operator
- `/deop <player>` -- revoke operator
- `/whitelist <add|remove|list|on|off>` -- whitelist management
- `/difficulty <peaceful|easy|normal|hard>` -- change difficulty
- `/gamerule <rule> <value>` -- set gamerule
- `/save-all` -- force world save
- `/stop` -- stop server gracefully
- `/say <message>` -- broadcast server message
- `/effect <player> <effect_id> [seconds] [amplifier]` -- apply potion effect
- `/clear <player> [item_id]` -- clear inventory
- `/xp <add|set> <player> <amount> [levels|points]` -- modify XP

### Gamerules to implement
- pvp (bool)
- keepInventory (bool)
- mobGriefing (bool)
- doFireTick (bool)
- doMobSpawning (bool)
- doDaylightCycle (bool)
- doWeatherCycle (bool)
- naturalRegeneration (bool)
- playersSleepingPercentage (int 0-100)
- maxEntityCramming (int)
- randomTickSpeed (int, default 3)
- showDeathMessages (bool)
- announceAdvancements (bool)

### Parsing
- Split command string by whitespace
- Match first token to command name
- Parse arguments by position
- Return error messages for wrong syntax
- Tab completion: send list of valid completions (player names, item names)

### Files to create
- `server/src/command/mod.rs` -- command dispatcher
- `server/src/command/admin.rs` -- operator commands
- `server/src/command/player.rs` -- player commands
- `server/src/command/permissions.rs` -- op list, permission checking

### Estimated: ~1000 lines
