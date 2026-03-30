# Void Mechanics

## What is the Void
- Empty space between and below islands
- No blocks, no terrain, just air/void
- Extends infinitely in all directions (X, Y, Z)
- Below all islands: void continues downward forever
- Above all islands: void continues upward (no build limit, just empty sky)

## Void Death
- Player falling below Y = -64: takes 4 HP/sec void damage (same as MC)
- At Y = -128: instant death regardless of HP
- Items dropped during void fall: destroyed if they reach Y = -128
- Items that land on blocks (bridge ledge, island edge): stay for 5 min
- Void damage bypasses armor (raw damage)
- Fire resistance, protection enchants do NOT reduce void damage

## Void Visual
- Looking down from island edge: dark gradient fading to deep blue/black
- Faint particles floating in void (small white/blue dots, very sparse)
- Deeper = darker: below Y=0 becomes progressively darker
- At night: void is pitch black, only island lights visible
- During aurora: void has faint colored reflections

## Void Sound
- Near island edge (within 5 blocks): subtle wind ambience
- Falling into void: escalating wind rush sound + heartbeat
- At Y < 0: deep rumbling bass sound
- Void ambience volume: scales with ring number (louder = more dangerous area)
- Adjustable in audio settings: "Void Ambience" slider

## Void Entities

### Dropped Items in Void
- Fall with normal gravity
- Destroyed at Y = -128
- If thrown from island edge: horizontal momentum carries them slightly
- Items on bridge that gets destroyed: fall into void

### Mobs in Void
- Mobs that fall off islands: die at Y = -128
- Mobs do not spawn in void (no solid blocks)
- Exception: ghasts (ring 10+) spawn in void air
- Ghasts are the only mob that "lives" in void

### Projectiles Through Void
- Arrows: fly normally, affected by gravity, fall into void
- Ender pearls: fly normally, affected by wind at high rings
- Fishing bobber in void: "sky fishing" mechanic (see unique_features.md)
- Snowballs, eggs: normal trajectory, lost in void if they miss

## Void and Water/Lava
- Water flowing off island edge: creates waterfall that renders down ~20 blocks then fades
- Water source block NOT consumed (permanent waterfall)
- Lava flowing off island edge: creates lava-fall, source IS consumed (lava drains away)
- Water/lava falling in void: does not create blocks, just visual effect
- Player in waterfall over void: can slow fall (water physics applies while in stream)
- Player can descend via waterfall to reach island underside, then climb back up

## Void and Light
- Void does not transmit light
- Torch on bridge: only lights the bridge, void below/beside remains dark
- Sky light from above passes through void normally (void is transparent to sky light)
- Block light from island: visible from void (glowing windows, torch-lit edges)
- Islands at night with no lights: nearly invisible from void (dangerous to bridge to)

## Building in Void
- Blocks can be placed in void if adjacent to existing block
- Bridges are "building in void" -- extending structures into empty space
- No block placement limit (can build infinitely into void if you have materials)
- Sand/gravel placed in void with no support: falls into void, destroyed
- Building below island: possible, creates hanging structures
- Building upward from island: possible, creates towers (useful for reaching higher islands)
