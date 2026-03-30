# Liquids

## Water
- Source block: placed by bucket, infinite source if 2+ adjacent sources flow into empty block
- Flows up to 7 blocks horizontally on flat surface
- Falls unlimited vertically, resets flow distance after falling
- Flow direction: picks shortest path to nearest edge/hole
- Swimming: player moves slower, can swim up by holding jump
- Breathing: player has 15 sec air supply (300 ticks), then takes 2 HP/sec drowning dmg
- Water breaks: torches, redstone dust, flowers, tall grass, crops on contact
- Water pushes entities in flow direction (~1.4 blocks/sec)
- Water extinguishes fire and burning entities
- Waterlogged blocks: slabs, stairs, fences, etc can contain water source
- Water + lava source = obsidian; water + lava flow = cobblestone; lava + water flow = stone
- Crops need water within 4 blocks to hydrate farmland
- Water color varies by biome
- Kelp, seagrass grow underwater
- Boats float on water, fastest transport on water (8 blocks/sec)
- Fishing: cast rod into water, wait 5-30 sec for bite, reel in for fish/junk/treasure

## Lava
- Source block: placed by bucket, NOT infinite (no infinite source trick)
- Flows 3 blocks horizontally in overworld (vs 7 in nether, but we have no nether)
- Falls unlimited vertically like water
- Burns entities: 4 HP/sec fire dmg + sets on fire for 15 sec after leaving
- Destroys most dropped items on contact
- Ignites flammable blocks within 1-2 blocks (wood, wool, leaves, etc)
- Light level 15 (brightest possible)
- Lava + water interactions: see water section above
- Entities sink in lava, very slow movement
- Fire resistance potion makes player immune to lava
- Can be used as fuel in furnace: 1 lava bucket smelts 100 items
- Lava flows much slower than water (30 ticks vs 5 ticks per spread)
