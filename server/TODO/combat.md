# TODO: Combat System

## Status: Partial (basic entity damage only)

## What exists
- `entity.rs`: take_damage() with invulnerability frames
- `player.rs`: take_damage() with simplified armor reduction
- `game.rs`: handle_entity_attack() -- basic damage, death, destroy entity

## What needs to be built

### Melee damage calculation
- Weapon damage by type (from items.json):
  - Hand: 1, Wood sword: 4, Stone: 5, Iron: 6, Diamond: 7
  - Axes: 7-9 depending on tier
  - Trident: 9
- Attack cooldown: 0.625 sec for sword, varies by weapon
- Full charge multiplier: 0.2 + 0.8 * (charge^2) -- partial charge = less damage
- Critical hit: falling + not on ground + full charge = +50% damage
- Sweep attack: sword full charge hits nearby mobs for 1 dmg + knockback

### Enchantment damage modifiers
- Sharpness I-V: +1/+1.5/+2/+2.5/+3
- Smite I-V: +2.5 per level vs undead (zombie, skeleton, drowned, etc)
- Bane of Arthropods I-V: +2.5 per level vs spiders/silverfish
- Fire Aspect I-II: set target on fire 4/8 sec
- Knockback I-II: extra knockback

### Armor damage reduction
- Each armor piece has defense points (from items.json)
- Total armor points: 0-20
- Armor toughness: diamond has 8 total
- Formula: damage * (1 - min(20, max(armor/5, armor - damage/(2+toughness/4))) / 25)
- Protection enchant: additional -4% per level per piece
- Fire/Blast/Projectile Protection: -8% per level for specific damage types
- Enchantment protection cap: 80% maximum reduction

### Knockback
- Base knockback on hit: push target 0.4 blocks away from attacker
- Sprint attack: +1 knockback level
- Knockback enchant: +0.5 blocks per level
- Vertical component: small upward boost (0.36 blocks/tick)
- Shield block: no knockback if blocking from front
- Knockback resistance: some mobs resist (iron golem, ravager)

### Ranged combat
- Bow: charge time 0-1 sec, damage 1-10 based on charge
  - Full charge + crit = 10 dmg
  - Consumes 1 arrow from inventory (unless Infinity enchant)
  - Power enchant: +25% per level
  - Punch: extra knockback on arrow hit
  - Flame: fire arrow, ignite target
- Crossbow: longer charge, holds charge when loaded
  - Damage: 6-11
  - Quick Charge enchant: faster loading
- Trident: throwable (8 dmg ranged), returns with Loyalty
  - Channeling: summon lightning during thunderstorm
  - Riptide: launch player in rain/water
- Snowball/egg: 0 dmg but knockback, snowball does 3 to blazes
- Splash potions: throw, AoE effect on impact

### Shield
- Block: hold right-click with shield in offhand
- Blocks 100% frontal damage (within 90 degree arc of look direction)
- Axe hit disables shield for 5 sec (100 ticks)
- Arrow blocked: arrow destroyed
- Explosion: reduces knockback and damage
- Durability consumed per blocked hit

### Invulnerability frames
- 10 ticks (0.5 sec) after taking damage
- During iframes: only damage higher than previous hit applies
- Prevents rapid-hit exploits

### Mob-specific combat
- Creeper: 1.5 sec fuse, 3-block explosion radius, charged = 2x
- Skeleton: shoots every 2 sec, accuracy varies by difficulty
- Enderman: teleports to dodge projectiles
- Spider: can spawn with potion effects on Hard
- Guardian: laser beam charges 2 sec then deals 6 dmg
- Evoker: summons vexes and fang attack
- Ghast: fireball reflectable by hitting

### Death & drops
- Player death: drop all inventory + XP at death location
- XP dropped: min(7 * level, 100) -- rest lost
- Items persist 5 min then despawn
- Mob death: drop loot table items + XP orbs
- Looting enchant: +1 max drop per level

### Files to create
- `server/src/combat/mod.rs` -- damage calculation dispatcher
- `server/src/combat/melee.rs` -- melee damage, cooldown, crits
- `server/src/combat/ranged.rs` -- bow, crossbow, trident, throwables
- `server/src/combat/armor.rs` -- armor reduction, enchant modifiers
- `server/src/combat/knockback.rs` -- knockback physics

### Estimated: ~2000 lines
