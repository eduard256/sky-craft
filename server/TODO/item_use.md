# TODO: Item Use System

## Status: Not implemented (stub in game.rs)

## What exists
- `game.rs`: handle_use_item() empty stub
- Protocol: UseItem packet defined

## What needs to be built

### Food eating
- Right-click with food item: start eating animation (32 ticks = 1.6 sec)
- If interrupted (take damage, switch item): cancel eat
- On finish: restore hunger + saturation, consume 1 item
- Food values loaded from common/data/foods.json
- Golden apple: Regen II 5s + Absorption 2min
- Enchanted golden apple: Regen V 20s + Absorption IV 2min + Resist I 5min + Fire Resist I 5min
- Raw chicken: 30% chance Hunger effect
- Rotten flesh: 80% chance Hunger effect
- Spider eye: Poison 4 sec
- Chorus fruit: random teleport (skip, End-specific)
- Honey bottle: 6 hunger + removes Poison
- Milk bucket: clears all effects

### Bucket
- Empty bucket + water source: pick up water, give water bucket
- Empty bucket + lava source: pick up lava, give lava bucket
- Water bucket on block: place water source
- Lava bucket on block: place lava source
- Bucket on cow: give milk bucket
- Bucket on fish: give fish bucket

### Bow
- Hold right-click: charge (0-1 sec)
- Release: shoot arrow entity
- Arrow consumed from inventory (or free with Infinity)
- Arrow damage based on charge time
- Arrow affected by gravity, sticks in blocks
- Arrow affected by wind at high rings

### Crossbow
- Hold right-click: load (1.25 sec, reduced by Quick Charge)
- Stays loaded until fired
- Click to fire loaded crossbow
- Same arrow mechanics as bow

### Trident
- Melee: 9 dmg
- Throw (right-click): 8 dmg ranged, flies in arc
- Loyalty enchant: returns after hitting
- Riptide: launch player (only in rain/water)
- Channeling: summon lightning on hit during thunder

### Fishing rod
- Right-click: cast bobber into water (or void for sky fishing)
- Wait for bite (5-30 sec in water, 15-45 sec in void)
- Right-click again: reel in, get fish/junk/treasure (water) or sky debris (void)
- Luck of the Sea enchant: better loot
- Lure enchant: faster bite time

### Throwables
- Snowball: throw, 0 dmg + knockback (3 dmg to blazes)
- Egg: throw, 1/8 chance spawn chicken
- Ender pearl: throw, teleport to landing spot, 5 dmg to self
- Ender pearl affected by wind
- Splash/lingering potion: throw, effect on impact

### Flint and steel
- Right-click on block: place fire on top face
- Right-click on TNT: ignite
- Right-click on creeper: ignite fuse
- Consumes 1 durability

### Shears
- Right-click on sheep: shear wool (1-3 wool, sheep becomes naked)
- Right-click on mooshroom: convert to cow + drop mushrooms
- Right-click on beehive (with campfire below): harvest honeycomb
- Break leaves/cobweb/vines faster with shears
- Break tall grass/fern to get item (instead of nothing)

### Hoe
- Right-click on dirt/grass: convert to farmland
- Right-click on coarse dirt: convert to dirt
- Right-click on rooted dirt: convert to dirt + drop roots

### Lead
- Right-click on mob: attach lead
- Right-click on fence: tie leashed mob to fence
- Leashed mob follows player within 10 blocks
- Lead breaks if distance > 10 blocks

### Name tag
- Use on anvil to set name, then right-click mob
- Named mob never despawns

### Saddle
- Right-click on horse/pig/strider: equip saddle
- Pig: controlled with carrot on stick when saddled

### Bone meal
- Right-click on crop: advance 2-5 growth stages
- Right-click on sapling: chance to grow tree
- Right-click on grass block: spawn tall grass + flowers
- Right-click on seagrass: grow tall seagrass

### Dye
- Right-click on sheep: dye sheep
- Combine with armor in crafting: dye leather armor
- Use on sign: change text color

### Map
- Right-click: create/update map of area
- Shows terrain in 128x128 block area
- Explored areas fill in as player moves

### Book and quill
- Right-click: open text editor
- Write text, sign to create written book

### Files to create
- `server/src/item/mod.rs` -- use_item dispatcher
- `server/src/item/food.rs` -- eating logic
- `server/src/item/bucket.rs` -- liquid pickup/place
- `server/src/item/ranged.rs` -- bow, crossbow, trident
- `server/src/item/throwable.rs` -- snowball, egg, ender pearl, potions
- `server/src/item/tool_use.rs` -- flint, shears, hoe, lead, name tag, bone meal

### Estimated: ~2000 lines
