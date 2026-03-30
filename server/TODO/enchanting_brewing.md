# TODO: Enchanting & Brewing

## Status: Not implemented

## What exists
- Protocol: enchanting table and brewing stand WindowType defined
- common/data/enchantments.json: 39 enchantments with names and IDs
- common/data/effects.json: 33 potion effects
- ItemStack: enchantments Vec<(u16, u8)>
- Player: active_effects Vec<PotionEffect>

## What needs to be built

### Enchanting table
- Open window when right-clicked
- Scan for bookshelves within 1 block (max 15 = level 30 enchants)
- Generate 3 enchantment options based on:
  - Player XP level
  - Number of bookshelves
  - Item type (determines valid enchantments)
  - Random seed (changes when any enchant applied)
- Show 1 guaranteed enchantment per option + "..." for hidden ones
- Cost: slot 1 = 1 level + 1 lapis, slot 2 = 2 + 2, slot 3 = 3 + 3
- Apply enchantment(s) to item, consume XP and lapis
- Enchantment conflict rules: Protection vs Fire/Blast/Projectile Protection, Silk Touch vs Fortune, Infinity vs Mending, etc

### Enchantment effects (apply during combat/mining/etc)
- Sharpness/Smite/Bane: add damage in combat system
- Efficiency: reduce break time in block_logic
- Fortune: multiply drops in loot_tables
- Silk Touch: change drop to block itself
- Unbreaking: chance to not consume durability
- Mending: XP repairs item instead of filling bar
- Protection/Fire/Blast/Projectile: reduce damage in combat
- Feather Falling: reduce fall damage in physics
- Respiration: extend breath timer underwater
- Aqua Affinity: normal mining speed underwater
- Depth Strider: faster underwater movement
- Thorns: reflect damage on attacker
- Looting: increase mob drops
- Power/Punch/Flame/Infinity: bow modifiers
- Knockback: increase knockback in combat
- Fire Aspect: set target on fire
- Sweeping Edge: increase sweep damage
- Loyalty/Channeling/Riptide: trident modifiers
- Quick Charge: faster crossbow loading

### Anvil
- Combine two items: merge enchantments + repair durability
- Apply enchanted book to item
- Rename item (custom_name in ItemStack)
- XP cost calculation: base + number of enchantments + prior work penalty
- Max cost: 39 levels (too expensive beyond)
- Anvil damage: 12% chance to degrade per use (3 stages then breaks)

### Grindstone
- Remove all enchantments from item, return some XP
- Combine 2 same items: durability = sum + 5% bonus
- Cannot remove curses (Curse of Vanishing, Curse of Binding)

### Brewing stand
- Fuel: blaze powder (20 brews per)
- Process: ingredient + 1-3 water bottles -> potion
- Brew time: 20 sec (400 ticks)
- Potion chain:
  1. Water Bottle + Nether Wart = Awkward Potion
  2. Awkward Potion + ingredient = Effect Potion
  3. Effect Potion + Redstone = Extended Duration
  4. Effect Potion + Glowstone = Enhanced Level
  5. Any Potion + Gunpowder = Splash Potion
- Key recipes:
  - Healing: glistering melon
  - Regen: ghast tear
  - Strength: blaze powder
  - Speed: sugar
  - Fire Resistance: magma cream
  - Night Vision: golden carrot
  - Water Breathing: pufferfish
  - Invisibility: fermented spider eye + night vision
  - Poison: spider eye
  - Harming: fermented spider eye + healing
  - Slowness: fermented spider eye + speed
  - Weakness: fermented spider eye
  - Slow Falling: phantom membrane
  - Void Resistance: void crystal shard (Sky Craft custom)

### Potion effect ticking
- Per-player per-tick: check active_effects
- Apply effect: speed modifier, damage modifier, regen/poison HP change
- Visual: send particles to other players
- Duration countdown, remove when expired
- Milk: clear all effects

### Files to create
- `server/src/enchanting/mod.rs` -- enchanting table logic
- `server/src/enchanting/anvil.rs` -- anvil combining
- `server/src/enchanting/effects.rs` -- enchantment effect application
- `server/src/brewing/mod.rs` -- brewing stand logic
- `server/src/brewing/recipes.rs` -- potion recipes
- `server/src/brewing/effects.rs` -- potion effect ticking

### Estimated: ~2000 lines
