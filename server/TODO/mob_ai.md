# TODO: Mob AI System

## Status: Not implemented

## What exists
- `entity.rs`: MobData struct, AIState enum (Idle/Wandering/Chasing/Attacking/Fleeing/etc)
- Basic despawn timer ticking in game loop
- Entity damage/death handling

## What needs to be built

### Pathfinding
- A* pathfinding on block grid (3D)
- Path cache per mob (recalculate every 20-40 ticks, not every tick)
- Avoid void edges (3 block buffer from island edge)
- Handle 1-block step-up (mobs can climb 1 block)
- Spiders: wall climbing pathfinding (vertical surfaces valid)
- Swimming: water blocks as valid but slow path nodes
- Max path length: 32 blocks (performance limit)

### State Machine per tick
```
Idle -> check for targets -> Chasing (if hostile + player in range)
Idle -> random timer -> Wandering (pick random nearby block)
Wandering -> reached target or timeout -> Idle
Chasing -> target in attack range -> Attacking
Chasing -> target lost (>32 blocks or line of sight broken) -> Idle
Attacking -> attack cooldown elapsed -> deal damage -> Attacking
Attacking -> target dead -> Idle
Any -> health < 50% + passive mob -> Fleeing
Fleeing -> distance > 16 blocks from threat -> Idle
```

### Per-mob behavior (40+ mob types)
Each mob needs unique:
- Spawn conditions (light level, block type, biome)
- Base HP, damage, speed, attack range
- Drop table (items + XP)
- Special abilities
- Sound triggers

### Priority mobs to implement first
1. Zombie (simplest hostile: walk toward player, attack melee)
2. Skeleton (ranged: strafe + shoot arrows)
3. Creeper (approach + fuse + explode)
4. Spider (climb walls, neutral in day)
5. Cow/Pig/Sheep/Chicken (wander, flee on hit, breed)
6. Enderman (teleport, neutral until looked at)

### Files to create
- `server/src/entity/ai/mod.rs` -- AI tick dispatcher
- `server/src/entity/ai/pathfinding.rs` -- A* implementation
- `server/src/entity/ai/hostile.rs` -- hostile mob behaviors
- `server/src/entity/ai/passive.rs` -- passive mob behaviors
- `server/src/entity/ai/neutral.rs` -- neutral mob behaviors

### Estimated: ~8000 lines
