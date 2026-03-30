# Chat & Commands

## Chat
- Open with T key, type message, Enter to send
- Visible to all players on server
- Messages show as: <nickname> message
- Chat history scrollable
- Max message length: 256 chars

## Commands (/ prefix)
- Only available to ops (operators) unless specified

### Player Commands (everyone)
- /help -- list available commands
- /msg <player> <text> -- private message
- /me <action> -- action message (*player1 waves*)
- /list -- show online players
- /spawn -- teleport to spawn (cooldown 5min)

### Operator Commands
- /gamemode <survival|creative|spectator> [player]
- /tp <player> <x y z> or /tp <player1> <player2>
- /give <player> <item> [amount]
- /time set <day|night|noon|value>
- /weather <clear|rain|thunder>
- /kill <player|entity>
- /ban <player> [reason]
- /unban <player>
- /kick <player> [reason]
- /op <player> -- grant operator
- /deop <player> -- revoke operator
- /whitelist <add|remove|list|on|off>
- /difficulty <peaceful|easy|normal|hard>
- /gamerule <rule> <value>
- /seed -- show world seed
- /save-all -- force save

## Game Rules
- pvp: true/false -- player vs player damage
- keepInventory: true/false -- keep items on death
- mobGriefing: true/false -- mobs break/place blocks
- doFireTick: true/false -- fire spreads
- doMobSpawning: true/false
- doDaylightCycle: true/false
- doWeatherCycle: true/false
- naturalRegeneration: true/false
- maxEntityCramming: 24 -- entities in 1 block before suffocation
- playersSleepingPercentage: 100 -- % needed to skip night
