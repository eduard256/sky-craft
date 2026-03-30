# Villages & Trading

## Village Generation
- Spawn on large islands (300+ blocks) in: Plains, Desert, Savanna, Taiga, Snowy Plains
- 3-10 buildings per village depending on island size
- Buildings: houses (beds), workstations, farms, wells, meeting point (bell)
- Roads connect buildings (path blocks or relevant surface)
- Architecture style matches biome (sandstone in desert, spruce in taiga, etc)

## Villager Professions
- Determined by nearby workstation block:
  - Armorer: blast furnace
  - Butcher: smoker
  - Cartographer: cartography table
  - Cleric: brewing stand
  - Farmer: composter
  - Fisherman: barrel
  - Fletcher: fletching table
  - Leatherworker: cauldron
  - Librarian: lectern
  - Mason/Stonemason: stonecutter
  - Shepherd: loom
  - Toolsmith: smithing table
  - Weaponsmith: grindstone
  - Nitwit: no profession, no trades
  - Unemployed: no workstation, will claim one if available

## Trading
- Right-click villager to open trade GUI
- Trade: give items -> receive items (usually emeralds as currency)
- Each profession has unique trade pool
- 5 levels: Novice -> Apprentice -> Journeyman -> Expert -> Master
- XP gained by trading, unlocks new tiers
- Prices affected by: supply/demand, reputation (curing, helping vs harming)
- Key trades:
  - Librarian: enchanted books (any enchant possible), bookshelves, name tags
  - Farmer: food, emeralds for crops
  - Armorer: diamond armor at Master level
  - Toolsmith: diamond tools at Master level
  - Cleric: ender pearls, redstone, glowstone, lapis
  - Cartographer: maps, item frames, banners
  - Weaponsmith: diamond sword/axe at Master level

## Village Mechanics
- Iron golem spawns at 10+ villagers with 21+ beds
- Villagers breed when beds + food surplus available
- Villagers flee from hostile mobs, run into houses at night
- Zombie siege: rare night event, zombies spawn inside village regardless of light
- Villager killed -> -1 reputation, trades get more expensive
- Zombie villager: cure with weakness potion + golden apple -> discounted trades

## Raids
- Triggered by player with Bad Omen entering village
- Bad Omen: killing pillager captain (banner carrier)
- 3-7 waves of pillagers, vindicators, evokers, witches, ravagers
- Between waves: short pause
- Raid victory: Hero of the Village effect (reduced trade prices for 2 days)
- Raid defeat: all villagers may be killed
- Loot: ominous banner, emeralds, saddle, totem of undying (from evoker)
