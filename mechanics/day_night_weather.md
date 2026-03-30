# Day/Night Cycle & Weather

## Day/Night Cycle
- Full cycle: 20 min real time (24000 ticks)
- Day: 0-12000 ticks (10 min), bright sky light
- Sunset: 12000-13000 (50 sec)
- Night: 13000-23000 (8.3 min), dark, hostile mobs spawn
- Sunrise: 23000-24000 (50 sec)
- Moon phases: 8 phases, affects slime spawn rate in swamp

## Sleeping
- Right-click bed at night or during thunderstorm
- All players on server must sleep to skip night (or majority with gamerule)
- Sets time to sunrise, clears weather
- Sets spawn point to bed location
- Resets phantom spawn timer
- Bed must have 2 air blocks above, not obstructed
- Can't sleep if hostile mob within 8 blocks ("You may not rest now, there are monsters nearby")
- Bed in wrong dimension = explode (not relevant, we have 1 dimension)

## Weather
- Clear: default, full sky light
- Rain: reduces sky light by 3, wets the world
  - Extinguishes fire on blocks
  - Fills cauldrons with water slowly
  - Endermen take dmg, teleport away
  - Wolves shake water off
  - Fishing bite time reduced (faster catches)
  - Duration: 0.5-1 day, random
- Thunderstorm: rain + lightning
  - Sky light drops to 0 (dark as night), hostile mobs can spawn
  - Lightning strikes randomly, sets fire, deals 5 dmg
  - Pig struck = zombie pigman (skip, no nether mobs? or convert to something else)
  - Villager struck = witch
  - Creeper struck = charged creeper
  - Trident with Channeling enchant: summons lightning on hit during thunder
  - Duration: 0.5-1 day, random
- Weather is server-controlled, same for all players
