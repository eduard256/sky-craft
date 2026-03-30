# Spawn & Respawn System

## Initial Spawn (First Join)

### Spawn Island Selection
- Server generates "spawn region" at world center (within ring 0)
- Algorithm finds suitable island: 100+ blocks wide, Plains or Forest biome
- Spawn point: center of island, on surface, at highest solid block
- If no suitable island in first 500 blocks: generate a forced spawn island
- Forced spawn island: guaranteed Plains, 200x200, with oak tree, water pond, basic ores

### Spawn Island Guarantees
- At least 1 oak tree (saplings for wood supply)
- At least 1 water source (infinite water creation possible)
- At least 1 exposed coal ore on surface/cliff face
- At least 1 iron ore within 10 blocks of surface
- Flat area for initial building
- Another island visible and reachable (within 50 blocks)

## Respawn (After Death)

### With Bed
- Player respawns at bed location
- Bed must still exist and not be obstructed (2 air blocks above)
- If bed destroyed (by mob, explosion, Anchor Break debuff): fall back to world spawn
- Notification: "Your bed was missing or obstructed" if bed invalid
- Respawn with empty inventory (items dropped at death location)

### Without Bed
- Respawn at world spawn point (ring 0 spawn island)
- This can be devastating for players deep in high rings
- All items at death location (may be in void = lost forever)

### Respawn Mechanics
- 5 second respawn delay (death screen visible)
- Respawn with full HP (20), full hunger (20), 0 XP
- 3 second invulnerability after respawn
- If respawning at world spawn while bed exists in high ring:
  compass still points to world spawn, not bed

## Death Location

### Item Drops
- All inventory items drop at death location as entities
- Items persist 5 minutes before despawning (standard MC)
- Items that fell into void: instantly destroyed, unrecoverable
- Items on bridge/island: stay for 5 min
- If player died to void (fell off): items may have scattered along fall path
  (some on bridge edge, some in void = partial recovery possible)

### Death Coordinates
- Death location XYZ shown on death screen
- Helps player navigate back to retrieve items
- If death was in void (Y < 0): coordinates shown are last position above Y=0

## Bed Mechanics

### Placement
- Standard MC bed placement rules
- Must be on solid block, 2 blocks long
- Can be placed on any island at any ring
- Strategic: players set beds at outpost islands before exploring deeper

### Sleeping
- Right-click bed at night or during thunderstorm
- All players on server must be in bed to skip night (or majority with gamerule)
- Sets spawn point to this bed
- Resets phantom timer
- Cannot sleep if hostile mob within 8 blocks
- Cannot sleep during Placement Lock or Mining Lock debuff

### Bed Destruction
- Bed broken by player: spawn point reverts to world spawn
- Bed broken by explosion (creeper, TNT): spawn point reverts
- Bed broken by Anchor Break mob debuff: spawn point reverts + notification
- Bed in unloaded chunk: still valid, chunk loaded on respawn

## Multi-Bed Strategy
- Experienced players carry multiple beds
- Place bed at each outpost island along the path
- Before entering dangerous island: place bed nearby on safe platform
- After looting: move bed forward or leave as checkpoint
- Loss of bed at high ring = potentially hours of bridge-building to return
