# Mob Combat Debuffs & Ring Scaling

## Overview
Starting from ring 3, hostile mobs gain special abilities that apply debuffs to players on hit.
Higher rings = more debuffs active simultaneously, stronger effects, longer durations.

## Placement Lock (Ring 3+)
- On hit: player cannot place blocks for 5 seconds after last hit
- HUD shows: "PLACEMENT BLOCKED" with countdown timer
- Prevents panic-bridging while under attack
- Critical on narrow bridges: mobs can corner you with no escape
- Duration scales: ring 3 = 3 sec, ring 10 = 5 sec, ring 20 = 8 sec

## Mining Lock (Ring 5+)
- On hit: player cannot break blocks for 4 seconds after last hit
- HUD shows: "MINING BLOCKED" with countdown timer
- Cannot dig escape routes while being attacked
- Combined with Placement Lock = player must fight or flee, can't build/dig
- Duration scales: ring 5 = 3 sec, ring 10 = 5 sec, ring 20 = 7 sec

## Inventory Lock (Ring 10+)
- On hit: player cannot open inventory/chests for 3 seconds
- HUD shows: "INVENTORY LOCKED" with countdown timer
- Prevents switching gear mid-combat at high rings
- Hotbar still works (can switch held item)
- Duration: fixed 3 sec regardless of ring

## Gravity Pull (Ring 8+)
- Some mobs (enderman, witch, evoker) pull player toward void edge
- 1-block pull toward nearest island edge on hit
- HUD shows: "GRAVITY SHIFT" with arrow showing pull direction
- Deadly on island edges: can pull player off cliff
- Only active within 10 blocks of island edge

## Fear (Ring 7+)
- On hit: player's view shakes violently for 2 seconds
- Screen edges darken, heartbeat sound plays
- Makes aiming difficult during the effect
- Applied by: creepers (pre-explosion), endermen, evokers
- Duration: ring 7 = 1 sec, ring 15 = 3 sec

## Void Sickness (Ring 15+)
- Applied when player takes damage while below island (hanging from bottom)
- Nausea + Slowness I for 10 seconds
- Makes exploring island undersides dangerous at high rings
- HUD shows: "VOID SICKNESS" with swirl effect

## Soul Drain (Ring 12+)
- Wither skeletons and evokers drain XP on hit
- Lose 1-3 XP levels per hit
- HUD shows: "SOUL DRAIN -N LEVELS"
- Incentivizes banking XP before deep-ring expeditions
- Drained XP is lost (not dropped as orbs)

## Anchor Break (Ring 20+)
- Very rare ability (5% of mobs at ring 20, scaling to 20% at ring 50)
- On hit: destroys player's bed spawn point remotely
- HUD shows: "YOUR BED WAS DESTROYED" (same as MC bed break message)
- Devastating: dying now means respawn at ring 0
- Player must set up new bed immediately
- Only melee mobs can apply this, not ranged

## Mob Ability Assignment by Ring
```
Ring 3-4:   50% of mobs have Placement Lock
Ring 5-6:   50% Placement Lock, 30% Mining Lock
Ring 7-9:   60% Placement Lock, 40% Mining Lock, 20% Fear
Ring 10-14: 70% PL, 50% ML, 30% Fear, 20% Inventory Lock, 10% Gravity Pull
Ring 15-19: 80% PL, 60% ML, 40% Fear, 30% IL, 20% GP, 10% Void Sickness
Ring 20-49: 90% PL, 70% ML, 50% Fear, 40% IL, 30% GP, 20% VS, 5% Anchor Break
Ring 50+:   All mobs have all debuffs, 20% Anchor Break, 10% Soul Drain
```

## Debuff Visual Indicators
- Each active debuff has a unique icon on HUD (left side of screen)
- Icons pulse when effect is about to expire
- Color coding: red = combat lock, purple = movement effect, yellow = warning
- Sound cue when debuff is applied (distinct per type)

## Mob Visual Changes by Ring
- Ring 5+: mob eyes glow brighter (red tint)
- Ring 10+: mobs have particle trail (dark smoke)
- Ring 20+: mobs slightly larger (1.1x scale)
- Ring 50+: mobs have flame particles around them
- Ring 100+: mobs leave brief shadow afterimages when moving
- These are client-side visual indicators of danger level
