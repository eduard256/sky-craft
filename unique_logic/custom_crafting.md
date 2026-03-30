# Sky Craft Custom Crafting Recipes

All standard MC recipes remain unchanged. These are ADDITIONAL recipes unique to Sky Craft.

## New Items

### Void Crystal Shard
- Source: mined from Void Crystal clusters on island undersides (ring 3+)
- Cannot be crafted, only found

### Void Resistance Potion
- Brewing: Awkward Potion + Void Crystal Shard
- Effect: immune to void damage for 30 sec
- Player still falls but takes no damage from void
- Does NOT prevent death at Y=-128 (instant kill still applies)
- Modifier: + redstone = 60 sec duration
- Splash variant: + gunpowder

### Void Crystal Block
- Recipe: 9 Void Crystal Shards (3x3)
- Light level 15
- Never decays on bridges (immune to bridge decay mechanic)
- Purple-tinted glowing block
- Can be broken and re-placed (drops itself)

### Void Compass
- Recipe: compass (center) + 4 Void Crystal Shards (cardinal positions)
- Points toward nearest unexplored island
- Range: 500 blocks
- Breaks after 100 uses (durability)

### Grappling Hook
- Recipe:
  ```
  [iron] [    ] [iron]
  [    ] [hook] [    ]
  [    ] [strg] [    ]
  ```
  (iron ingot, tripwire hook, string)
- Actually: 3 iron ingots (top row) + 1 tripwire hook (center) + 2 string (bottom center, bottom)
- Durability: 64 uses
- Right-click to throw, hooks to blocks within 20 blocks
- Pulls player to hook point at 8 blocks/sec

### Island Anchor
- Recipe:
  ```
  [obs ] [iron] [obs ]
  [iron] [diam] [iron]
  [obs ] [iron] [obs ]
  ```
  (4 obsidian corners, 4 iron blocks sides, 1 diamond center)
- Place on island: prevents bridge decay within 200 block radius
- One per island
- Emits light beam upward (like beacon)

### Sky Lantern
- Recipe:
  ```
  [papr] [papr] [papr]
  [papr] [trch] [papr]
  [    ] [papr] [    ]
  ```
  (5 paper + 1 torch = 4 sky lanterns)
- Throwable, floats upward, emits light for 5 min

### Bridge Rail
- Recipe:
  ```
  [iron] [    ] [iron]
  [iron] [    ] [iron]
  [iron] [stck] [iron]
  ```
  (6 iron ingots + 1 stick = 16 bridge rails)
- Place on bridge edges, blocks wind push, thinner than fence

### Weather Station
- Recipe:
  ```
  [    ] [glas] [    ]
  [iron] [clck] [iron]
  [rdst] [comp] [rdst]
  ```
  (1 glass + 2 iron + 1 clock + 1 compass + 2 redstone dust)
- Shows weather and wind forecast
- Text display on block face

### Emergency Recall
- Recipe:
  ```
  [eprl] [gold] [eprl]
  [gold] [bed ] [gold]
  [eprl] [gold] [eprl]
  ```
  (4 ender pearls + 4 gold ingots + 1 bed)
- One-time use, teleports to bed spawn
- Drops all items at current location before teleport

### Void Binoculars
- Recipe:
  ```
  [    ] [gold] [    ]
  [glas] [    ] [glas]
  [    ] [iron] [    ]
  ```
  (2 glass panes + 1 gold ingot + 2 iron ingots -- actually arranged differently)
- Correction -- simpler recipe:
  ```
  [iron] [    ] [iron]
  [    ] [gold] [    ]
  [glas] [    ] [glas]
  ```
- Zoom 4x, reveals distant islands
- Unlimited durability

### Wind Charm
- Recipe:
  ```
  [strg] [fthr] [strg]
  [fthr] [eprl] [fthr]
  [strg] [fthr] [strg]
  ```
  (4 feathers + 4 string + 1 ender pearl)
- Held in offhand: reduces wind push by 50%
- Stacks with armor wind reduction and sneaking reduction
- Durability: 500 uses (1 use per wind tick while held)

## Modified Vanilla Recipes

### Elytra (Future)
- NOT found in End (no End dimension)
- Craft recipe (expensive):
  ```
  [phan] [phan] [phan]
  [phan] [diam] [phan]
  [strg] [    ] [strg]
  ```
  (5 phantom membranes + 1 diamond + 2 string)
- Much more expensive than finding in End city
- Critical for high-ring traversal
- Repaired with phantom membrane (standard MC)

### Nether Materials Access
- Netherrack: found on volcanic islands, not craftable
- Soul Sand: found on volcanic islands, not craftable
- Nether Wart: found in volcanic island loot chests / soul sand patches
- Blaze Rod: dropped by blazes on volcanic islands
- Ghast Tear: dropped by ghasts in void (ring 10+)
- Magma Cream: dropped by magma cubes on volcanic islands
- Glowstone: found in volcanic island structures, craftable from 4 glowstone dust
- All nether materials exist but must be found, not accessed via portal
