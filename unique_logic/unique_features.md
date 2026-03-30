# Sky Craft Unique Features

## 1. Void Echo
- When player is near island edge (within 3 blocks of void), sounds echo
- Mob sounds, footsteps, block breaking all get reverb effect
- Echo intensity increases in deeper void (further from nearest island)
- Falling into void: wind rushing sound that gets louder, pitch drops
- Creates eerie atmosphere unique to floating island world

## 2. Island Magnetism
- Each island has weak "gravity field" extending 5 blocks beyond its edge
- Within this field: fall speed reduced by 30%
- Gives player slightly more reaction time if they slip off edge
- Visual: faint particle shimmer at island edges
- Does NOT prevent death (still fall into void eventually, just slower start)
- Does NOT affect horizontal movement or wind push

## 3. Sky Fishing
- Cast fishing rod off island edge into void
- Instead of fish: catches "sky debris" -- random items that float up from below
- Loot table: string, feathers, phantom membrane, glowstone dust, prismarine shards
- Rare catches: enchanted books, name tags, saddles
- Bite time: longer than water fishing (15-45 sec)
- Only works at island edges facing void (not on water)
- Rod must dangle into void (below island bottom Y)

## 4. Cloud Platforms
- Semi-solid cloud blocks spawn in void between islands (ring 2+)
- Appear as white/gray translucent platforms (3-8 blocks wide)
- Player can stand on them for 5 seconds, then slowly sink through (1 block/sec)
- After 8 total seconds: fall through completely
- Reset after player leaves for 10 sec
- Great as stepping stones between islands if bridge not built yet
- Mobs cannot stand on clouds (fall through immediately)
- Cannot place blocks on clouds (they're not solid enough)
- Spawn pattern: noise-based, deterministic with seed, typically 2-5 between nearby islands

## 5. Updrafts
- Vertical air columns between some islands (ring 3+)
- Visible as swirling white particle columns, 3-5 blocks diameter
- Player entering updraft: pushed upward at 5 blocks/sec
- Allows reaching higher islands without building stairs
- Updraft positions are fixed (generated with world seed)
- Some updrafts lead to nowhere (just push you up then you fall)
- Smart players build platforms at updraft tops to catch themselves
- Updraft strength varies: some weak (slow lift), some strong (launch)
- Can carry items and mobs upward too

## 6. Void Wells
- Rare downward vortexes in void (ring 5+)
- Visible as dark swirling particles pulling inward and down
- Pull radius: 8 blocks
- Pull strength: 2-4 blocks/sec toward center
- Player caught in well: dragged down into void unless they escape
- Wells are stationary, can be marked and avoided
- Building bridge through a well: blocks placed inside are slowly destroyed (1 per 30 sec)
- Players must build bridges AROUND void wells
- Wells emit deep humming sound audible from 20 blocks

## 7. Aurora Events (Ring 5+)
- Random event: aurora appears in sky for 5-10 min
- Beautiful visual: colored light ribbons across sky (green, blue, purple)
- During aurora: all mobs are passive (stop attacking, wander aimlessly)
- Mob debuffs are suspended during aurora
- Wind calms to zero
- Perfect window for dangerous bridge building or resource runs
- Frequency: once per 2-3 game days, unpredictable timing
- Notification: "The Aurora appears..." when it starts
- Warning: "The Aurora is fading..." 30 sec before end
- Mobs return to hostile immediately when aurora ends

## 8. Resonance Mining
- Hitting an ore block causes nearby same-type ores to briefly glow through stone
- Effect radius: 5 blocks
- Duration: 3 seconds of glow
- Only visible to the player who hit the ore
- Helps find ore veins without strip-mining
- Works on all ore types
- Glowing ores emit particles visible through walls
- No gameplay advantage beyond visibility (still need to mine through stone)

## 9. Island Anchoring
- Craft "Island Anchor" block: 4 obsidian + 4 iron blocks + 1 diamond
- Place on island surface: prevents Bridge Decay on all bridges connected to this island
- Range: 200 blocks from anchor
- One anchor per island, breaks if island not visited for 30 days
- Expensive to craft but saves bridge maintenance
- Visual: slowly rotating obsidian frame with diamond core
- Emits beacon-like light beam upward (visible from far away)
- Can be seen through void fog

## 10. Resource Scarcity Adaptation
- Islands that have been heavily mined: ores regenerate VERY slowly
- 1 random ore block regenerates per island per real-time hour
- Only ores that were originally present can regenerate
- Diamond never regenerates
- This prevents total resource depletion but doesn't replace proper exploration
- Applies only to ore blocks, not to placed/crafted blocks

## 11. Bridge Markers
- Right-click a placed block with empty hand while sneaking: place invisible marker
- Marker shows as small colored dot on map and minimap
- 8 colors available (cycle with repeated right-clicks)
- Use for: marking safe paths, danger zones, points of interest
- Limit: 100 markers per player
- Visible only to the player who placed them
- Can be removed by sneaking + right-click again

## 12. Void Compass
- Crafted: compass + ender pearl
- Points toward nearest unexplored island (no player-placed blocks on it)
- Useful for finding new islands to explore and settle
- Range: 500 blocks
- If all islands within range explored: spins randomly

## 13. Sky Lanterns
- Crafted: paper + torch
- Throwable item that floats upward slowly (0.5 blocks/sec)
- Emits light level 12 while floating
- Lasts 5 minutes then burns out
- Used to illuminate void during bridge building
- Can be thrown horizontally to light up path ahead
- Multiple lanterns create beautiful floating light display

## 14. Grappling Hook (Ring 5+ Craftable)
- Crafted: 3 iron ingots + 2 string + 1 tripwire hook
- Right-click to throw (20 block range)
- Hooks onto solid blocks, pulls player toward hook point
- Travel speed: 8 blocks/sec (faster than walking)
- Cannot hook onto air/void
- Durability: 64 uses
- Essential tool for high-ring traversal
- Can save player from falling: throw at island while falling, get pulled to safety
- Cannot be used during Placement Lock debuff

## 15. Void Crystals
- Rare resource found only on island undersides (hanging from bottom)
- Glowing purple crystal clusters (use amethyst texture with purple tint)
- Must be mined from below (dangerous: player hangs from island bottom)
- Drops: 1-3 Void Crystal Shards
- Uses:
  - 4 shards + bottle = Void Resistance Potion (immune to void damage for 30 sec)
  - 9 shards = Void Crystal Block (light level 15, never decays on bridges)
  - 2 shards + compass = Void Compass (see above)
- Only spawn on islands in ring 3+
- Very rare: 1-3 clusters per island

## 16. Weather Station Block
- Crafted: 4 iron + 1 clock + 1 glass + 2 redstone dust + 1 compass
- Place on island surface
- Shows weather forecast for next 10 min: clear, rain, thunder, void fog
- Shows wind forecast: direction and strength trend
- Critical for planning bridge building in high rings
- Text display on block face (like sign text)
- Updates every 30 seconds

## 17. Emergency Recall
- Craftable item: 4 ender pearls + 4 gold ingots + 1 bed
- One-time use
- Instantly teleports player to their bed spawn point
- 1 second casting time (hold right-click), interrupted by damage
- Drops all items at current location before teleporting
- Player arrives at bed with empty inventory
- Lifesaver when trapped in high ring with no escape
- Very expensive to craft, carry sparingly

## 18. Island Naming Signpost
- Craft: sign + map + iron nugget
- Place on island: shows island's procedural name floating above the sign
- Name visible from 100 blocks away (like beacon text)
- Helps identify islands from distance during exploration
- Only 1 per island

## 19. Bridge Rails
- Craft: 6 iron ingots + 1 stick = 16 bridge rails
- Place on bridge edges (like fence but thinner)
- Prevents wind from pushing player off bridge (100% wind block at rail)
- Mobs can still walk past rails
- Cheaper than full fence + requires less material than enclosed bridge
- Visual: thin iron bar railing, looks modern and clean

## 20. Void Binoculars
- Craft: 2 glass + 1 gold ingot + 2 iron ingots
- Right-click: zoom view 4x (like spyglass/optifine zoom)
- Shows island outlines at extreme distance (render normally hidden islands)
- Temporarily increases render distance to 32 chunks while looking through
- Essential for scouting bridge routes before committing materials
- Durability: unlimited (not consumed)
