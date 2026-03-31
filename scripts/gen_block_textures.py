#!/usr/bin/env python3
"""
Generate common/data/block_textures.json from blocks.json and available texture files.

Rules:
- Key = minStateId (default state for the block)
- Only base texture files (no numbered variants like cobblestone1.png)
- Multi-face patterns detected by suffix: _top, _side, _bottom, _front, _back, _end
- Logs: base=side texture, _top=top+bottom
- Unknown blocks (no textures found): textures=null
- transparent field taken from blocks.json
- Overrides: manually specified exceptions where texture name != block name
"""

import json
import os
import sys

BLOCKS_JSON = "/home/user/sky-craft/common/data/blocks.json"
TEXTURES_DIR = "/home/user/sky-craft/client/assets/textures/minecraft/textures/block"
OUTPUT = "/tmp/block_textures.json"

# Load texture files (only base files, no numbered variants like cobblestone1.png)
all_files = set()
for f in os.listdir(TEXTURES_DIR):
    if f.endswith(".png"):
        all_files.add(f)

def tex_exists(name):
    return name in all_files

def base_only(name):
    """Only pick texture without numeric suffix."""
    return f"{name}.png" if tex_exists(f"{name}.png") else None

# =========================================================================
# OVERRIDES: blocks where texture name != block name, or special face rules
# Format: block_name -> textures dict (or None for no-texture blocks)
# =========================================================================
OVERRIDES = {
    # Air and special "no-render" blocks
    "air":         None,
    "void_air":    None,
    "cave_air":    None,
    "moving_piston": None,
    "piston_head": None,
    "bubble_column": None,
    "barrier":     None,
    "structure_void": None,

    # Water / lava - animated, special handling
    "water":       {"all": "water_still.png"},
    "lava":        {"all": "lava_still.png"},

    # grass_block: bottom = dirt (not grass_block_bottom)
    "grass_block": {"top": "grass_block_top.png", "side": "grass_block_side.png", "bottom": "dirt.png"},
    # podzol: bottom = dirt
    "podzol":      {"top": "podzol_top.png", "side": "podzol_side.png", "bottom": "dirt.png"},
    # mycelium: bottom = dirt
    "mycelium":    {"top": "mycelium_top.png", "side": "mycelium_side.png", "bottom": "dirt.png"},

    # grass_path / dirt_path - renamed
    "grass_path":  {"top": "grass_path_top.png", "side": "grass_path_side.png", "bottom": "dirt.png"},
    "dirt_path":   {"top": "grass_path_top.png", "side": "grass_path_side.png", "bottom": "dirt.png"},

    # Sandstone: top/bottom are different, sides = base sandstone
    "sandstone":   {"top": "sandstone_top.png", "side": "sandstone.png", "bottom": "sandstone_bottom.png"},
    "red_sandstone": {"top": "red_sandstone_top.png", "side": "red_sandstone.png", "bottom": "red_sandstone_bottom.png"},

    # Logs -- bark on all sides variant (oak_wood = all bark)
    "oak_wood":          {"all": "oak_log.png"},
    "spruce_wood":       {"all": "spruce_log.png"},
    "birch_wood":        {"all": "birch_log.png"},
    "jungle_wood":       {"all": "jungle_log.png"},
    "acacia_wood":       {"all": "acacia_log.png"},
    "dark_oak_wood":     {"all": "dark_oak_log.png"},
    "stripped_oak_wood":      {"all": "stripped_oak_log.png"},
    "stripped_spruce_wood":   {"all": "stripped_spruce_log.png"},
    "stripped_birch_wood":    {"all": "stripped_birch_log.png"},
    "stripped_jungle_wood":   {"all": "stripped_jungle_log.png"},
    "stripped_acacia_wood":   {"all": "stripped_acacia_log.png"},
    "stripped_dark_oak_wood": {"all": "stripped_dark_oak_log.png"},

    # Cherry / Mangrove / Bamboo -- these textures are missing from assets, fallback to similar
    "cherry_planks":     {"all": "dark_oak_planks.png"},
    "mangrove_planks":   {"all": "jungle_planks.png"},
    "bamboo_planks":     {"all": "oak_planks.png"},
    "bamboo_mosaic":     {"all": "oak_planks.png"},
    "cherry_log":        {"top": "birch_log_top.png", "side": "birch_log.png"},
    "cherry_wood":       {"all": "birch_log.png"},
    "mangrove_log":      {"top": "jungle_log_top.png", "side": "jungle_log.png"},
    "mangrove_wood":     {"all": "jungle_log.png"},
    "mangrove_roots":    {"all": "jungle_log.png"},
    "muddy_mangrove_roots": {"all": "dirt.png"},
    "bamboo_block":      {"top": "bamboo_stalk.png", "side": "bamboo_stalk.png"},
    "stripped_cherry_log":   {"top": "stripped_birch_log_top.png", "side": "stripped_birch_log.png"},
    "stripped_mangrove_log": {"top": "stripped_jungle_log_top.png", "side": "stripped_jungle_log.png"},
    "stripped_bamboo_block": {"all": "bamboo_stalk.png"},
    "stripped_cherry_wood":   {"all": "stripped_birch_log.png"},
    "stripped_mangrove_wood": {"all": "stripped_jungle_log.png"},
    "cherry_leaves":     {"all": "acacia_leaves.png"},
    "mangrove_leaves":   {"all": "jungle_leaves.png"},
    "cherry_sapling":    {"all": "birch_sapling.png"},

    # Deepslate ores -- overlay on deepslate
    "deepslate_gold_ore":      {"all": "stone.png"},
    "deepslate_iron_ore":      {"all": "stone.png"},
    "deepslate_coal_ore":      {"all": "stone.png"},
    "deepslate_lapis_ore":     {"all": "stone.png"},
    "deepslate_diamond_ore":   {"all": "stone.png"},
    "deepslate_redstone_ore":  {"all": "stone.png"},
    "deepslate_emerald_ore":   {"all": "stone.png"},
    "deepslate_copper_ore":    {"all": "stone.png"},

    # Snow block -- missing, use snow texture
    "snow_block":        {"all": "snow.png"},

    # Bookshelf -- sides = bookshelf, top/bottom = oak_planks
    "bookshelf":         {"top": "oak_planks.png", "side": "bookshelf.png", "bottom": "oak_planks.png"},

    # Crafting table
    "crafting_table":    {"top": "crafting_table_top.png", "side": "crafting_table_side.png",
                          "front": "crafting_table_front.png", "bottom": "oak_planks.png"},

    # Furnace family -- front face only when idle
    "furnace":           {"top": "furnace_top.png", "side": "furnace_side.png", "front": "furnace_front.png", "bottom": "furnace_top.png"},
    "blast_furnace":     {"top": "blast_furnace_top.png", "side": "blast_furnace_side.png", "front": "blast_furnace_front.png", "bottom": "blast_furnace_top.png"},
    "smoker":            {"top": "smoker_top.png", "side": "smoker_side.png", "front": "smoker_front.png", "bottom": "smoker_bottom.png"},

    # Dispenser / Dropper
    "dispenser":         {"top": "furnace_top.png", "side": "furnace_side.png", "front": "dispenser_front.png", "bottom": "furnace_top.png"},
    "dropper":           {"top": "furnace_top.png", "side": "furnace_side.png", "front": "dropper_front.png", "bottom": "furnace_top.png"},

    # TNT
    "tnt":               {"top": "tnt_top.png", "side": "tnt_side.png", "bottom": "tnt_bottom.png"},

    # Piston
    "piston":            {"top": "piston_top.png", "side": "piston_side.png", "bottom": "piston_bottom.png"},
    "sticky_piston":     {"top": "piston_top_sticky.png", "side": "piston_side.png", "bottom": "piston_bottom.png"},

    # Quartz block
    "quartz_block":      {"top": "quartz_block_top.png", "side": "quartz_block_side.png", "bottom": "quartz_block_bottom.png"},
    "chiseled_quartz_block": {"top": "chiseled_quartz_block_top.png", "side": "chiseled_quartz_block.png", "bottom": "chiseled_quartz_block_top.png"},
    "quartz_pillar":     {"top": "quartz_pillar_top.png", "side": "quartz_pillar.png", "bottom": "quartz_pillar_top.png"},

    # Hay block
    "hay_block":         {"top": "hay_block_top.png", "side": "hay_block_side.png", "bottom": "hay_block_top.png"},

    # Bee nest / beehive
    "bee_nest":          {"top": "bee_nest_top.png", "side": "bee_nest_side.png", "front": "bee_nest_front.png", "bottom": "bee_nest_bottom.png"},
    "beehive":           {"top": "beehive_end.png", "side": "beehive_side.png", "front": "beehive_front.png", "bottom": "beehive_end.png"},

    # Basalt / polished basalt
    "basalt":            {"top": "basalt_top.png", "side": "basalt_side.png", "bottom": "basalt_top.png"},
    "polished_basalt":   {"top": "polished_basalt_top.png", "side": "polished_basalt_side.png", "bottom": "polished_basalt_top.png"},

    # Bone block
    "bone_block":        {"top": "bone_block_top.png", "side": "bone_block_side.png", "bottom": "bone_block_top.png"},

    # Pumpkin / carved pumpkin / jack o lantern
    "pumpkin":           {"top": "pumpkin_top.png", "side": "pumpkin_side.png", "bottom": "pumpkin_top.png"},
    "carved_pumpkin":    {"top": "pumpkin_top.png", "side": "carved_pumpkin.png", "bottom": "pumpkin_top.png"},
    "jack_o_lantern":    {"top": "pumpkin_top.png", "side": "jack_o_lantern.png", "bottom": "pumpkin_top.png"},
    "melon":             {"top": "melon_top.png", "side": "melon_side.png", "bottom": "melon_top.png"},

    # End portal frame
    "end_portal_frame":  {"top": "end_portal_frame_top.png", "side": "end_portal_frame_side.png", "bottom": "end_stone.png"},

    # Nether stuff
    "nether_wart_block": {"all": "nether_wart_block.png"},
    "warped_nylium":     {"top": "warped_nylium.png", "side": "warped_nylium_side.png", "bottom": "netherrack.png"},
    "crimson_nylium":    {"top": "crimson_nylium.png", "side": "crimson_nylium_side.png", "bottom": "netherrack.png"},

    # Cactus
    "cactus":            {"top": "cactus_top.png", "side": "cactus_side.png", "bottom": "cactus_bottom.png"},

    # Mud / packed mud / mud bricks
    "mud":               {"all": "dirt.png"},
    "packed_mud":        {"all": "dirt.png"},
    "mud_bricks":        {"all": "bricks.png"},

    # Deepslate family
    "deepslate":         {"all": "stone.png"},
    "infested_deepslate":{"all": "stone.png"},
    "smooth_basalt":     {"all": "basalt_side.png"},

    # Doors -- not solid, skip
    "oak_door":     None,
    "spruce_door":  None,
    "birch_door":   None,
    "jungle_door":  None,
    "acacia_door":  None,
    "dark_oak_door":None,
    "cherry_door":  None,
    "mangrove_door":None,
    "bamboo_door":  None,
    "iron_door":    None,
    "crimson_door": None,
    "warped_door":  None,

    # Stairs, slabs, fences, walls, pressure plates, buttons -- non-solid mesh, skip textures for now
    # (renderer will need special handling, but we still want to assign base texture)
    # These are handled by auto-detection below since they often have no matching files

    # Signs, torches, ladders, rails -- not cube blocks
    "torch":         None,
    "wall_torch":    None,
    "soul_torch":    None,
    "soul_wall_torch": None,
    "redstone_torch": None,
    "redstone_wall_torch": None,
    "ladder":        None,
    "rail":          None,
    "powered_rail":  None,
    "detector_rail": None,
    "activator_rail":None,
    "oak_sign":      None,
    "spruce_sign":   None,
    "birch_sign":    None,
    "jungle_sign":   None,
    "acacia_sign":   None,
    "dark_oak_sign": None,
    "cherry_sign":   None,
    "mangrove_sign": None,
    "bamboo_sign":   None,
    "crimson_sign":  None,
    "warped_sign":   None,
    "oak_wall_sign": None,
    "spruce_wall_sign": None,
    "birch_wall_sign": None,
    "jungle_wall_sign": None,
    "acacia_wall_sign": None,
    "dark_oak_wall_sign": None,
    "cherry_wall_sign": None,
    "mangrove_wall_sign": None,
    "bamboo_wall_sign": None,
    "crimson_wall_sign": None,
    "warped_wall_sign": None,
    "oak_hanging_sign": None,
    "spruce_hanging_sign": None,
    "birch_hanging_sign": None,
    "jungle_hanging_sign": None,
    "acacia_hanging_sign": None,
    "dark_oak_hanging_sign": None,
    "cherry_hanging_sign": None,
    "mangrove_hanging_sign": None,
    "bamboo_hanging_sign": None,
    "crimson_hanging_sign": None,
    "warped_hanging_sign": None,
    "oak_wall_hanging_sign": None,
    "spruce_wall_hanging_sign": None,
    "birch_wall_hanging_sign": None,
    "jungle_wall_hanging_sign": None,
    "acacia_wall_hanging_sign": None,
    "dark_oak_wall_hanging_sign": None,
    "cherry_wall_hanging_sign": None,
    "mangrove_wall_hanging_sign": None,
    "bamboo_wall_hanging_sign": None,
    "crimson_wall_hanging_sign": None,
    "warped_wall_hanging_sign": None,

    # Beds -- not cube
    "white_bed":   None, "orange_bed":  None, "magenta_bed": None, "light_blue_bed": None,
    "yellow_bed":  None, "lime_bed":    None, "pink_bed":    None, "gray_bed":        None,
    "light_gray_bed": None, "cyan_bed": None, "purple_bed":  None, "blue_bed":        None,
    "brown_bed":   None, "green_bed":   None, "red_bed":     None, "black_bed":       None,

    # Banners
    "white_banner": None, "orange_banner": None, "magenta_banner": None, "light_blue_banner": None,
    "yellow_banner": None, "lime_banner": None, "pink_banner": None, "gray_banner": None,
    "light_gray_banner": None, "cyan_banner": None, "purple_banner": None, "blue_banner": None,
    "brown_banner": None, "green_banner": None, "red_banner": None, "black_banner": None,
    "white_wall_banner": None, "orange_wall_banner": None, "magenta_wall_banner": None,
    "light_blue_wall_banner": None, "yellow_wall_banner": None, "lime_wall_banner": None,
    "pink_wall_banner": None, "gray_wall_banner": None, "light_gray_wall_banner": None,
    "cyan_wall_banner": None, "purple_wall_banner": None, "blue_wall_banner": None,
    "brown_wall_banner": None, "green_wall_banner": None, "red_wall_banner": None,
    "black_wall_banner": None,

    # Skulls / heads
    "skeleton_skull": None, "skeleton_wall_skull": None,
    "wither_skeleton_skull": None, "wither_skeleton_wall_skull": None,
    "zombie_head": None, "zombie_wall_head": None,
    "player_head": None, "player_wall_head": None,
    "creeper_head": None, "creeper_wall_head": None,
    "dragon_head": None, "dragon_wall_head": None,
    "piglin_head": None, "piglin_wall_head": None,

    # Potted plants -- skip (flower_pot rendering)
    "potted_torchflower": None, "potted_oak_sapling": None, "potted_spruce_sapling": None,
    "potted_birch_sapling": None, "potted_jungle_sapling": None, "potted_acacia_sapling": None,
    "potted_cherry_sapling": None, "potted_dark_oak_sapling": None, "potted_mangrove_propagule": None,
    "potted_fern": None, "potted_dandelion": None, "potted_poppy": None,
    "potted_blue_orchid": None, "potted_allium": None, "potted_azure_bluet": None,
    "potted_red_tulip": None, "potted_orange_tulip": None, "potted_white_tulip": None,
    "potted_pink_tulip": None, "potted_oxeye_daisy": None, "potted_cornflower": None,
    "potted_lily_of_the_valley": None, "potted_wither_rose": None, "potted_red_mushroom": None,
    "potted_brown_mushroom": None, "potted_dead_bush": None, "potted_cactus": None,
    "potted_bamboo": None, "potted_azalea_bush": None, "potted_flowering_azalea_bush": None,
    "potted_crimson_fungus": None, "potted_warped_fungus": None,
    "potted_crimson_roots": None, "potted_warped_roots": None,
    "potted_mangrove_propagule": None,

    # Carpets -- flat, no cube
    "white_carpet": None, "orange_carpet": None, "magenta_carpet": None, "light_blue_carpet": None,
    "yellow_carpet": None, "lime_carpet": None, "pink_carpet": None, "gray_carpet": None,
    "light_gray_carpet": None, "cyan_carpet": None, "purple_carpet": None, "blue_carpet": None,
    "brown_carpet": None, "green_carpet": None, "red_carpet": None, "black_carpet": None,

    # Misc non-cube
    "fire":          None,
    "soul_fire":     None,
    "end_portal":    None,
    "end_gateway":   None,
    "wheat":         None,
    "beetroots":     None,
    "carrots":       None,
    "potatoes":      None,
    "nether_wart":   None,
    "cocoa":         None,
    "bamboo":        None,
    "campfire":      None,
    "soul_campfire": None,
    "sweet_berry_bush": None,
    "frosted_ice":   None,
    "seagrass":      None,
    "tall_seagrass": None,
    "redstone_wire": None,
    "tripwire":      None,
    "tripwire_hook": None,
    "lever":         None,
    "chest":         None,
    "trapped_chest": None,
    "ender_chest":   None,
    "cobweb":        None,
    "piston_head":   None,
    "moving_piston": None,
    "spawner":       None,
    "fern":          None,
    "dead_bush":     None,
    "sunflower":     None,
    "lilac":         None,
    "peony":         None,
    "rose_bush":     None,
    "tall_grass":    None,
    "large_fern":    None,
    "sugar_cane":    None,
    "kelp":          None,
    "kelp_plant":    None,
    "vine":          None,
    "glow_lichen":   None,
    "sculk_vein":    None,
    "scaffolding":   None,
    "torchflower":   None,
    "torchflower_crop": None,
    "bamboo_sapling": None,
    "hanging_roots": None,
    "cave_vines":    None,
    "cave_vines_plant": None,
    "spore_blossom": None,
    "big_dripleaf":  None,
    "big_dripleaf_stem": None,
    "small_dripleaf":None,
    "frogspawn":     None,
    "pink_petals":   None,
    "mangrove_propagule": None,
    "suspicious_sand": None,
    "powder_snow":   None,
    "light":         None,
    "structure_block": None,
    "jigsaw":        None,
    "moving_piston": None,
    "piston_head":   None,
    "bubble_column": None,
}


def detect_textures(name):
    """Auto-detect texture pattern for a block by name."""
    b = f"{name}.png"
    top = f"{name}_top.png"
    side = f"{name}_side.png"
    bottom = f"{name}_bottom.png"
    front = f"{name}_front.png"
    end = f"{name}_end.png"

    has_b = tex_exists(b)
    has_top = tex_exists(top)
    has_side = tex_exists(side)
    has_bot = tex_exists(bottom)
    has_front = tex_exists(front)
    has_end = tex_exists(end)

    # Log-type: base (side) + top (top+bottom)
    if has_b and has_top and not has_side:
        return {"top": top, "side": b, "bottom": top}

    # Full 6-face with front
    if has_front and has_side and has_top and has_bot:
        return {"top": top, "side": side, "front": front, "bottom": bottom}

    # Furnace-type: front + side + top (no bottom file -> use top)
    if has_front and has_side and has_top and not has_bot:
        return {"top": top, "side": side, "front": front, "bottom": top}

    # Dispenser-type: only front (no side file)
    if has_front and not has_side and not has_top:
        return {"all": front}

    # top + side + bottom
    if has_top and has_side and has_bot:
        return {"top": top, "side": side, "bottom": bottom}

    # top + side (no bottom -> use side for bottom, or top for bottom -- pumpkin style)
    if has_top and has_side and not has_bot:
        return {"top": top, "side": side, "bottom": top}

    # Only top (rare)
    if has_top and not has_side and not has_b:
        return {"all": top}

    # end suffix (like nether quartz)
    if has_end and has_b:
        return {"top": end, "side": b, "bottom": end}

    # Simple: one texture all faces
    if has_b:
        return {"all": b}

    return None


# =========================================================================
# Main generation
# =========================================================================
with open(BLOCKS_JSON) as f:
    blocks = json.load(f)

result = {}
stats = {"all": 0, "multi": 0, "null": 0, "fallback": 0}

for block in blocks:
    name = block["name"]
    state_id = str(block["minStateId"])
    transparent = block.get("transparent", False)
    bounding = block.get("boundingBox", "block")

    entry = {
        "name": name,
        "transparent": transparent,
    }

    if name in OVERRIDES:
        entry["textures"] = OVERRIDES[name]
        if OVERRIDES[name] is None:
            stats["null"] += 1
        elif len(OVERRIDES[name]) == 1 and "all" in OVERRIDES[name]:
            stats["all"] += 1
        else:
            stats["multi"] += 1
    else:
        textures = detect_textures(name)
        entry["textures"] = textures
        if textures is None:
            stats["null"] += 1
            stats["fallback"] += 1
        elif len(textures) == 1 and "all" in textures:
            stats["all"] += 1
        else:
            stats["multi"] += 1

    result[state_id] = entry

with open(OUTPUT, "w") as f:
    json.dump(result, f, indent=2)

print(f"Generated {len(result)} entries")
print(f"  single texture (all):    {stats['all']}")
print(f"  multi-face:              {stats['multi']}")
print(f"  null (non-cube/missing): {stats['null']}")
print(f"  auto-null (no match):    {stats['fallback']}")

