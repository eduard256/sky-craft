# Mob Spawning

## General Rules
- Mobs spawn in chunks within 128 blocks of player, despawn beyond 128 blocks
- Mobs at 32-128 blocks: random chance to despawn each tick
- Mobs within 32 blocks: never despawn naturally
- Named mobs (name tag) never despawn
- Mob cap per player: 70 hostile, 10 passive, 15 ambient, 5 water
- Spawn attempts every tick for hostile, every 400 ticks for passive
- Mobs need valid spawn space: 2 blocks tall air for most, 1 for small mobs

## Hostile Mob Spawning (Night / Dark)
- Spawn on solid blocks at light level 0 (block light, not sky light)
- At night: sky light drops, surface becomes valid spawn area
- In caves/dark rooms: spawn anytime regardless of day/night
- Min 24 blocks from any player

## Passive Mob Spawning
- Spawn on grass blocks at light level 9+
- Only spawn during world generation and rarely after
- Animals persist once spawned (don't despawn unless killed)
- Breed cooldown: 5 min after breeding

## Sky Craft Special Rules
- Mobs can fall off island edges into void (die at Y < 0)
- Spawn only on island surfaces, not in void/air
- Passive mobs spawn on grass-covered islands during generation
- Hostile mobs follow standard light-level rules on islands
