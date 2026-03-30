# TODO: Respawn System

## Status: Death handled, respawn not implemented

## What exists
- `game.rs`: kill_player() sets is_dead, sends DeathInfo
- `player.rs`: respawn_position() calculates bed or world spawn
- No respawn request handling from client

## What needs to be built

### Death flow
1. Player HP reaches 0 -> kill_player() called
2. Send DeathInfo packet (cause, position, score)
3. Client shows death screen with "Respawn" button
4. Client sends respawn request (new packet or reuse existing)
5. Server receives respawn request:
   - Reset player HP to 20, food to 20, saturation to 5
   - Clear inventory (items already dropped at death location)
   - Clear active effects and debuffs
   - Calculate respawn position (bed or world spawn)
   - Check bed validity (still exists, not obstructed)
   - If bed invalid: fall back to world spawn, notify "Your bed was missing or obstructed"
   - Teleport player to respawn position
   - Send Respawn packet (resets client state)
   - Send new chunks around respawn position
   - Send PlayerPositionAndLook
   - Set invulnerability ticks = 60 (3 sec)
   - Resume normal play

### Item drops on death
- Drop all inventory items at death position as item entities
- Items fall with gravity, land on surface or into void
- XP dropped: min(7 * level, 100) points as XP orb entities
- Keep inventory gamerule: skip dropping
- Items in void (death Y < 0): spawn at last valid Y position above void

### Bed mechanics
- Place bed: standard block placement
- Sleep attempt (right-click bed at night):
  - Check: is it night or thunderstorm?
  - Check: no hostile mob within 8 blocks?
  - Check: bed not obstructed (2 air blocks above)?
  - If all pass: set spawn point, start sleep animation
  - If all players sleeping (or % threshold): skip to morning
  - Reset phantom timer
- Bed destroyed: spawn point reverts to world spawn

### Anchor Break debuff (Sky Craft)
- When applied: server destroys player's bed block
- Player notified: "Your bed was destroyed by a dark force"
- Next death: respawn at world spawn (potentially ring 0)

### Files to create/modify
- Add respawn packet to protocol (C2SRespawn or reuse Login)
- Modify `game.rs`: handle_respawn_request()
- Modify `player.rs`: reset_on_death()
- Modify block placement: track bed positions per player

### Estimated: ~500 lines
