#!/usr/bin/env bash
# Downloads and extracts game assets from open-source repositories.
# Run from the repository root: ./scripts/download_assets.sh
#
# Sources:
#   Textures: PixelPerfectionCE (CC-BY-SA-4.0)
#   Sounds:   VoxeLibre / MineClone2 (GPL-3.0)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

TEXTURES_DIR="$ROOT_DIR/client/assets/textures"
SOUNDS_DIR="$ROOT_DIR/client/assets/sounds"
TMP_DIR="$ROOT_DIR/.asset_cache"

TEXTURES_REPO="https://github.com/Athemis/PixelPerfectionCE.git"
SOUNDS_REPO="https://github.com/VoxeLibre/VoxeLibre.git"

mkdir -p "$TEXTURES_DIR" "$SOUNDS_DIR" "$TMP_DIR"

# ─── Textures ────────────────────────────────────────────────────────────────

if [ -f "$TEXTURES_DIR/.downloaded" ]; then
    echo "[textures] Already downloaded, skipping. Delete $TEXTURES_DIR/.downloaded to re-download."
else
    echo "[textures] Cloning PixelPerfectionCE..."
    TEXTURES_TMP="$TMP_DIR/PixelPerfectionCE"
    if [ ! -d "$TEXTURES_TMP" ]; then
        git clone --depth 1 "$TEXTURES_REPO" "$TEXTURES_TMP"
    fi

    echo "[textures] Copying texture files..."
    # Copy all PNG textures, preserving directory structure relative to assets/
    find "$TEXTURES_TMP/assets" -name "*.png" | while read -r file; do
        rel="${file#$TEXTURES_TMP/assets/}"
        dest_dir="$TEXTURES_DIR/$(dirname "$rel")"
        mkdir -p "$dest_dir"
        cp "$file" "$dest_dir/"
    done

    count=$(find "$TEXTURES_DIR" -name "*.png" | wc -l)
    echo "[textures] Copied $count texture files."
    touch "$TEXTURES_DIR/.downloaded"
fi

# ─── Sounds ──────────────────────────────────────────────────────────────────

if [ -f "$SOUNDS_DIR/.downloaded" ]; then
    echo "[sounds] Already downloaded, skipping. Delete $SOUNDS_DIR/.downloaded to re-download."
else
    echo "[sounds] Cloning VoxeLibre (sparse, sounds only)..."
    SOUNDS_TMP="$TMP_DIR/VoxeLibre"
    if [ ! -d "$SOUNDS_TMP" ]; then
        git clone --depth 1 --filter=blob:none --sparse "$SOUNDS_REPO" "$SOUNDS_TMP"
        cd "$SOUNDS_TMP"
        git sparse-checkout set "mods/CORE/mcl_sounds/sounds" \
                               "mods/CORE/vl_env_sounds/sounds" \
                               "mods/ENTITIES/mobs_mc/sounds" \
                               "mods/ENTITIES/mcl_mobs/sounds" \
                               "mods/ENVIRONMENT" \
                               "mods/ITEMS/mcl_amethyst/sounds" \
                               "mods/ITEMS/mcl_armor/sounds" \
                               "mods/ITEMS/mcl_barrels/sounds" \
                               "mods/ITEMS/mcl_bells/sounds" \
                               "mods/ITEMS/mcl_bows/sounds" \
                               "mods/ITEMS/mcl_brewing/sounds" \
                               "mods/ITEMS/mcl_chests/sounds" \
                               "mods/ITEMS/mcl_core/sounds" \
                               "mods/ITEMS/mcl_doors/sounds" \
                               "mods/ITEMS/mcl_enchanting/sounds" \
                               "mods/ITEMS/mcl_fences/sounds" \
                               "mods/ITEMS/mcl_fire/sounds" \
                               "mods/ITEMS/mcl_fishing/sounds" \
                               "mods/ITEMS/mcl_jukebox/sounds" \
                               "mods/ITEMS/mcl_mud/sounds" \
                               "mods/ITEMS/mcl_potions/sounds" \
                               "mods/ITEMS/mcl_sculk/sounds" \
                               "mods/ITEMS/mcl_shields/sounds" \
                               "mods/ITEMS/mcl_throwing/sounds" \
                               "mods/ITEMS/mcl_tnt/sounds" \
                               "mods/ITEMS/mcl_tools/sounds" \
                               "mods/PLAYER/mcl_criticals/sounds" \
                               "mods/PLAYER/mcl_hunger/sounds" \
                               "mods/PLAYER/mcl_music/sounds" \
                               "mods/PLAYER/vl_cavesounds/sounds" \
                               "mods/HUD/mcl_experience/sounds" \
                               "mods/HUD/awards/sounds"
        cd "$ROOT_DIR"
    fi

    echo "[sounds] Extracting .ogg files into categorized directories..."

    # Map source directories to our sound categories
    declare -A SOUND_CATEGORIES=(
        ["mods/CORE/mcl_sounds/sounds"]="blocks"
        ["mods/CORE/vl_env_sounds/sounds"]="environment"
        ["mods/ENTITIES/mobs_mc/sounds"]="mobs"
        ["mods/ENTITIES/mcl_mobs/sounds"]="mobs"
        ["mods/ENVIRONMENT/lightning/sounds"]="environment"
        ["mods/ENVIRONMENT/mcl_weather/sounds"]="environment"
        ["mods/ITEMS/mcl_amethyst/sounds"]="blocks"
        ["mods/ITEMS/mcl_armor/sounds"]="items"
        ["mods/ITEMS/mcl_barrels/sounds"]="blocks"
        ["mods/ITEMS/mcl_bells/sounds"]="blocks"
        ["mods/ITEMS/mcl_bows/sounds"]="combat"
        ["mods/ITEMS/mcl_brewing/sounds"]="items"
        ["mods/ITEMS/mcl_chests/sounds"]="blocks"
        ["mods/ITEMS/mcl_core/sounds"]="blocks"
        ["mods/ITEMS/mcl_doors/sounds"]="blocks"
        ["mods/ITEMS/mcl_enchanting/sounds"]="items"
        ["mods/ITEMS/mcl_fences/sounds"]="blocks"
        ["mods/ITEMS/mcl_fire/sounds"]="environment"
        ["mods/ITEMS/mcl_fishing/sounds"]="items"
        ["mods/ITEMS/mcl_jukebox/sounds"]="music"
        ["mods/ITEMS/mcl_mud/sounds"]="blocks"
        ["mods/ITEMS/mcl_potions/sounds"]="items"
        ["mods/ITEMS/mcl_sculk/sounds"]="blocks"
        ["mods/ITEMS/mcl_shields/sounds"]="combat"
        ["mods/ITEMS/mcl_throwing/sounds"]="combat"
        ["mods/ITEMS/mcl_tnt/sounds"]="environment"
        ["mods/ITEMS/mcl_tools/sounds"]="items"
        ["mods/PLAYER/mcl_criticals/sounds"]="combat"
        ["mods/PLAYER/mcl_hunger/sounds"]="player"
        ["mods/PLAYER/mcl_music/sounds"]="music"
        ["mods/PLAYER/vl_cavesounds/sounds"]="environment"
        ["mods/HUD/mcl_experience/sounds"]="player"
        ["mods/HUD/awards/sounds"]="player"
    )

    for src_path in "${!SOUND_CATEGORIES[@]}"; do
        category="${SOUND_CATEGORIES[$src_path]}"
        full_src="$SOUNDS_TMP/$src_path"
        dest="$SOUNDS_DIR/$category"
        mkdir -p "$dest"
        if [ -d "$full_src" ]; then
            find "$full_src" -name "*.ogg" -exec cp {} "$dest/" \;
        fi
    done

    count=$(find "$SOUNDS_DIR" -name "*.ogg" | wc -l)
    echo "[sounds] Extracted $count sound files into categories:"
    for cat_dir in "$SOUNDS_DIR"/*/; do
        [ -d "$cat_dir" ] || continue
        cat_name=$(basename "$cat_dir")
        cat_count=$(find "$cat_dir" -name "*.ogg" | wc -l)
        echo "  $cat_name: $cat_count files"
    done
    touch "$SOUNDS_DIR/.downloaded"
fi

# ─── Cleanup ─────────────────────────────────────────────────────────────────

echo ""
echo "Done. Assets are in:"
echo "  Textures: $TEXTURES_DIR"
echo "  Sounds:   $SOUNDS_DIR"
echo ""
echo "To force re-download, delete the .downloaded marker files and re-run."
