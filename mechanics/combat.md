# Combat

## Melee Attack
- Attack cooldown: 0.625 sec (sword), varies by weapon
- Cooldown indicator shows on HUD, full charge = max dmg
- Sweeping attack (sword): hits nearby mobs in arc for 1 dmg + knockback
- Critical hit: attack while falling = +50% dmg, particle stars
- Knockback: base knockback on hit, increased by sprinting (+1 level)

## Weapon Damage (base, full charge)
- Hand: 1
- Wood sword: 4, axe: 7
- Stone sword: 5, axe: 9
- Iron sword: 6, axe: 9
- Diamond sword: 7, axe: 9
- Trident: 9 (melee + throwable)

## Ranged
- Bow: 1-10 dmg depending on charge time (1 sec full charge = max)
- Full charge + crit = 10 dmg, arrow trails particles
- Crossbow: 6-11 dmg, longer charge but holds charge when loaded
- Trident (thrown): 8 dmg, returns with Loyalty enchant
- Snowball/egg: 0 dmg but knockback, snowball does 3 dmg to blazes

## Armor
- Each piece has armor points (total 0-20, shown as chestplate icons)
- Leather: 7 total, iron: 15, diamond: 20, gold: 11
- Armor toughness: diamond has 8, reduces high-dmg attacks more
- Dmg reduction formula: armor_points / 25 (roughly)
- Shield: blocks 100% frontal dmg when active, 5 sec cooldown if hit by axe

## Armor Durability
- Helmet: leather 55, iron 165, diamond 363, gold 77
- Chest: leather 80, iron 240, diamond 528, gold 112
- Legs: leather 75, iron 225, diamond 495, gold 105
- Boots: leather 65, iron 195, diamond 429, gold 91
- Shield: 336

## Invulnerability Frames
- After taking dmg: 0.5 sec invulnerable (10 ticks)
- Prevents rapid-hit exploits
- During iframes, only higher dmg than previous hit applies

## Death
- Player dies at 0 HP, drops all inventory + XP
- Items persist on ground for 5 min then despawn
- Respawn at spawn island (or bed if set)
- XP dropped = min(7 * level, 100), rest is lost
- Void death (fall below world): same as normal death but items may be lost in void
