# Environmental Hazards

## Wind System

### Overview
Wind exists in the void between islands. Intensity scales with ring number.
Wind only affects entities NOT standing on solid blocks (on bridges, falling, jumping).

### Wind Mechanics
- Wind direction: changes every 30-120 sec (random, seeded per chunk + time)
- Wind strength: 0 at ring 0, increases per ring (see formula in rings_progression.md)
- Wind only active in "open air" (no solid block within 3 blocks in any horizontal direction)
- Standing on island surface (surrounded by blocks): no wind effect
- Standing on 1-block-wide bridge over void: full wind effect
- Sneaking reduces wind push by 70%
- Wearing heavy armor (iron+) reduces wind push by 30%

### Wind Effects
- Pushes player horizontally in wind direction
- Pushes dropped items
- Pushes arrows mid-flight (affects aim at high rings)
- Pushes mobs on bridges (mobs can fall off too!)
- Does NOT push blocks or placed items

### Wind Gusts
- Random sudden bursts stronger than base wind
- Duration: 1-3 seconds
- Strength: 2-5x base wind
- HUD warning: "WIND GUST" flashes 0.5 sec before gust hits
- Sound: loud whoosh sound, directional
- At ring 10+: gusts can push player 3-5 blocks sideways
- At ring 50+: gusts can push player 10+ blocks

### Wind Visual
- Particle streaks in wind direction (white/gray streaks)
- Cloud movement speed matches wind direction
- Tree leaves on island edges sway in wind direction
- Stronger wind = more dense particles

## Void Lightning (Ring 10+)

### Overview
Lightning strikes in the void between islands. Does not strike island surfaces
(regular thunderstorm lightning handles that). Void lightning targets bridges and
players in open air.

### Mechanics
- Strikes random locations in void every 30-90 sec during thunderstorms
- At ring 10-20: strikes random void locations (mostly miss players)
- At ring 30+: 20% chance of targeting nearest player in void/on bridge
- At ring 50+: 50% chance of targeting player
- Damage: 8 HP + fire 5 sec (same as regular lightning)
- Destroys 1-3 blocks where it strikes (can break bridges!)
- HUD warning: "VOID LIGHTNING" + crackling sound 1 sec before strike
- Lightning rod on bridge: attracts lightning, protects nearby blocks

### Visual
- Purple-tinted lightning bolt (vs white regular lightning)
- Illuminates void briefly, showing island undersides
- Thunder sound echoes longer in void (reverb effect)

## Void Fog (Ring 5+)

### Overview
Dense fog rolls through void at certain times, reducing visibility drastically.

### Mechanics
- Occurs randomly for 2-5 min, every 10-30 min
- Render distance reduced to 3 chunks during fog
- Islands beyond fog are invisible
- Mobs in fog have +50% chance of spawning as invisible (Invisibility I)
- Navigation becomes very difficult
- Compass and map still work normally
- HUD shows: "VOID FOG" with current visibility percentage

### Ring Scaling
- Ring 5-10: fog is light, visibility reduced to 5 chunks
- Ring 10-20: moderate fog, 3 chunks visibility
- Ring 30+: dense fog, 1-2 chunks visibility
- Ring 50+: near-zero visibility, can barely see past 8 blocks

## Falling Debris (Ring 15+)

### Overview
Blocks occasionally fall from above through the void. Remnants from destroyed
islands above (lore: the sky world is slowly crumbling).

### Mechanics
- Random falling block entities spawn high above islands
- Fall through void, can land on islands or bridges
- Block types: cobblestone, stone, gravel, sand
- Damage on hit: 3-8 HP depending on block type
- Gravel/sand land and stay, stone/cobble break on impact
- Frequency: 1-3 per minute per loaded chunk in high rings
- Warning: shadow on ground 1 sec before impact
- Can be dodged by moving, sneaking under overhangs

## Void Whispers (Ring 20+)

### Overview
Audio-only hazard. Strange sounds play when near void edges, designed to
disorient and distract players during dangerous crossings.

### Mechanics
- Random ambient sounds: footsteps behind player, mob sounds that don't exist,
  block breaking sounds, explosion sounds in distance
- Only play when player is within 5 blocks of void (island edge or on bridge)
- Volume increases with ring number
- Not real threats, purely psychological
- Can be disabled in audio settings ("Void Ambience" slider)

## Island Tremors (Ring 25+)

### Overview
Islands in high rings occasionally shake, simulating instability.

### Mechanics
- Random tremor every 5-15 min on occupied islands
- Duration: 2-5 seconds
- Camera shake effect (client-side)
- Loose blocks (sand, gravel) can fall during tremor
- Items on ground bounce slightly
- Mobs and players get brief Slowness I during tremor
- HUD shows: "ISLAND TREMOR"
- No structural damage to placed blocks (player-placed blocks are safe)
- Only natural sand/gravel affected

## Sky Darkening (Ring 10+)

### Overview
Higher rings have progressively darker sky, even during day.

### Mechanics
- Ring 0: normal MC sky
- Ring 5: slightly muted colors, sun dimmer
- Ring 10: sky has gray tint, effective sky light reduced by 2
- Ring 20: perpetual dusk feeling, sky light -4
- Ring 50: nearly dark sky even at noon, sky light -8
- Ring 100+: permanent night-like sky, only block light matters
- Affects hostile mob spawning: at ring 20+, some can spawn during day
- Stars visible during day at ring 30+
- Moon visible during day at ring 50+
