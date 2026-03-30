# Hunger & Health

## Health
- Max 20 HP (10 hearts), each heart = 2 HP
- Natural regen: 1 HP/4 sec when hunger >= 18 (9 shanks)
- Rapid regen: at hunger = 20, heals 1 HP/0.5 sec (costs 6 saturation/HP)
- At hunger 0: takes 1 HP/4 sec starvation dmg (easy: stops at 10HP, normal: 1HP, hard: death)

## Hunger System
- Hunger bar: 0-20 (10 shanks)
- Saturation: hidden value 0-20, depletes before hunger bar
- Exhaustion: hidden counter, when reaches 4.0 -> loses 1 saturation (or 1 hunger if sat=0)
- Exhaustion sources:
  - Sprinting: 0.1/meter
  - Jumping: 0.05/jump, sprint-jump: 0.2
  - Swimming: 0.01/meter
  - Mining block: 0.005/block
  - Taking dmg: 0.1/hit
  - Attacking: 0.1/swing
  - Hunger effect: 0.1/tick at level I

## Food Values (hunger + saturation restored)
- Steak/cooked porkchop: 8 hunger, 12.8 sat (best common food)
- Cooked chicken: 6, 7.2
- Bread: 5, 6.0
- Cooked cod: 5, 6.0
- Cooked salmon: 6, 9.6
- Apple: 4, 2.4
- Golden apple: 4, 9.6 + Regen II 5sec + Absorption 2min
- Enchanted golden apple: 4, 9.6 + Regen II 20sec + Absorption IV 2min + Resist I 5min + Fire Resist I 5min
- Carrot: 3, 3.6
- Baked potato: 5, 6.0
- Raw meat: less hunger, raw chicken gives 30% chance Hunger effect
- Rotten flesh: 4, 0.8 + 80% chance Hunger effect 30sec
- Spider eye: 2, 3.2 + Poison 4sec
- Suspicious stew: 6, 7.2 + random potion effect
- Cake: 14 total (7 slices x 2 hunger), placed as block
- Cookie: 2, 0.4
- Pumpkin pie: 8, 4.8
- Melon slice: 2, 1.2
- Sweet berries: 2, 0.4
- Honey bottle: 6, 1.2 + removes Poison

## Damage Types & Amounts
- Fall damage: (distance - 3) HP. Fall 4 blocks = 1 HP, 23 blocks = 20 HP (lethal)
- Fire: 1 HP/sec standing in fire
- Lava: 4 HP/sec
- Drowning: 2 HP/sec after air runs out
- Suffocation (in blocks): 1 HP/sec
- Void (below Y=-64): 4 HP/sec
- Cactus: 1 HP on contact
- Starvation: 1 HP/4sec
- Lightning: 5 HP
- Explosion: up to 65 HP (point blank creeper)
