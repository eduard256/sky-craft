// World generator. Converts chunk positions into chunk sections filled with blocks.
// Each chunk is generated deterministically from the world seed.

use skycraft_protocol::types::*;

use super::island::{self, IslandDef};

/// Block state IDs for common blocks. These must match common/data/blocks.json.
/// In v0.0.1 we use simplified IDs; full mapping loaded from JSON later.
pub mod blocks {
    use skycraft_protocol::types::BlockStateId;

    pub const AIR: BlockStateId = 0;
    pub const STONE: BlockStateId = 1;
    pub const GRANITE: BlockStateId = 2;
    pub const DIORITE: BlockStateId = 4;
    pub const ANDESITE: BlockStateId = 6;
    pub const GRASS_BLOCK: BlockStateId = 8;
    pub const DIRT: BlockStateId = 10;
    pub const COBBLESTONE: BlockStateId = 14;
    pub const OAK_PLANKS: BlockStateId = 15;
    pub const OAK_LOG: BlockStateId = 126;
    pub const SPRUCE_LOG: BlockStateId = 129;
    pub const BIRCH_LOG: BlockStateId = 132;
    pub const JUNGLE_LOG: BlockStateId = 135;
    pub const ACACIA_LOG: BlockStateId = 138;
    pub const CHERRY_LOG: BlockStateId = 141;
    pub const DARK_OAK_LOG: BlockStateId = 144;
    pub const MANGROVE_LOG: BlockStateId = 147;
    pub const OAK_LEAVES: BlockStateId = 233;
    pub const SPRUCE_LEAVES: BlockStateId = 261;
    pub const BIRCH_LEAVES: BlockStateId = 289;
    pub const JUNGLE_LEAVES: BlockStateId = 317;
    pub const ACACIA_LEAVES: BlockStateId = 345;
    pub const CHERRY_LEAVES: BlockStateId = 373;
    pub const DARK_OAK_LEAVES: BlockStateId = 401;
    pub const MANGROVE_LEAVES: BlockStateId = 429;
    pub const SAND: BlockStateId = 66;
    pub const GRAVEL: BlockStateId = 68;
    pub const COAL_ORE: BlockStateId = 36;
    pub const IRON_ORE: BlockStateId = 37;
    pub const COPPER_ORE: BlockStateId = 38;
    pub const GOLD_ORE: BlockStateId = 39;
    pub const DIAMOND_ORE: BlockStateId = 41;
    pub const LAPIS_ORE: BlockStateId = 43;
    pub const REDSTONE_ORE: BlockStateId = 45;
    pub const EMERALD_ORE: BlockStateId = 47;
    pub const WATER: BlockStateId = 34;
    pub const LAVA: BlockStateId = 35;
    pub const BEDROCK: BlockStateId = 33;
    pub const DEEPSLATE: BlockStateId = 12;
    pub const SNOW_BLOCK: BlockStateId = 155;
    pub const ICE: BlockStateId = 174;
    pub const CLAY: BlockStateId = 120;
    pub const MYCELIUM: BlockStateId = 248;
    pub const NETHERRACK: BlockStateId = 249;
    pub const SOUL_SAND: BlockStateId = 250;
    pub const OBSIDIAN: BlockStateId = 251;
    pub const SANDSTONE: BlockStateId = 67;
    pub const MOSSY_COBBLESTONE: BlockStateId = 253;
    pub const GOLD_BLOCK: BlockStateId = 255;
    pub const DIAMOND_BLOCK: BlockStateId = 256;
    pub const EMERALD_BLOCK: BlockStateId = 257;
    pub const IRON_BLOCK: BlockStateId = 258;
    pub const MAGMA_BLOCK: BlockStateId = 259;
    pub const BASALT: BlockStateId = 260;
    pub const PACKED_ICE: BlockStateId = 261;
    pub const BLUE_ICE: BlockStateId = 262;
}

type TreeBlocks = Vec<(i32, i32, i32, BlockStateId)>;

/// World generator produces chunk sections from seed + position.
pub struct WorldGenerator {
    seed: i64,
    pub flat: bool,
    tree_cache: std::sync::Mutex<std::collections::HashMap<(i32, i32), Vec<TreeBlocks>>>,
}

impl WorldGenerator {
    pub fn new(seed: i64) -> Self {
        Self { seed, flat: false, tree_cache: std::sync::Mutex::new(std::collections::HashMap::new()) }
    }

    /// Generate a chunk section at the given chunk position.
    pub fn generate_chunk(&self, chunk_pos: ChunkPos) -> ChunkSection {
        if self.flat {
            return self.generate_flat_chunk(chunk_pos);
        }
        // Find all islands that might intersect this chunk
        let nearby = island::islands_near_chunk(chunk_pos.x, chunk_pos.z, self.seed);

        let mut section = ChunkSection::empty();
        let mut has_blocks = false;

        // World block range covered by this chunk
        let base_x = chunk_pos.x * 16;
        let base_y = chunk_pos.y * 16;
        let base_z = chunk_pos.z * 16;

        for island in &nearby {
            // Quick AABB check: does this island possibly overlap this chunk?
            if !island_overlaps_chunk(island, base_x, base_y, base_z) {
                continue;
            }

            // Generate blocks for each position in the chunk
            for ly in 0..16u8 {
                for lz in 0..16u8 {
                    for lx in 0..16u8 {
                        let wx = base_x + lx as i32;
                        let wy = base_y + ly as i32;
                        let wz = base_z + lz as i32;

                        if let Some(block) = self.block_at(wx, wy, wz, island) {
                            if block != blocks::AIR {
                                set_block_in_section(&mut section, lx, ly, lz, block);
                                has_blocks = true;
                            }
                        }
                    }
                }
            }
        }

        if has_blocks { section } else { ChunkSection::empty() }
    }

    /// Determine what block should be at a specific world coordinate for a given island.
    fn block_at(&self, x: i32, y: i32, z: i32, island: &IslandDef) -> Option<BlockStateId> {
        let surface_y = island.surface_y(x, z, self.seed)?;
        let bottom_y = island.bottom_y(x, z, self.seed)?;

        // Outside vertical range of island
        if y > surface_y || y < bottom_y {
            return None;
        }

        let depth_from_surface = (surface_y - y) as u32;
        let total_depth = (surface_y - bottom_y).max(1) as u32;
        let depth_fraction = depth_from_surface as f64 / total_depth as f64;

        // Surface layer (biome-specific)
        if depth_from_surface == 0 {
            return Some(self.surface_block(island));
        }

        // Topsoil (2-4 blocks below surface)
        if depth_from_surface <= 3 {
            return Some(self.topsoil_block(island));
        }

        // Check for ores
        if let Some(ore) = self.try_place_ore(x, y, z, depth_fraction, island) {
            return Some(ore);
        }

        // Check for precious blocks at high rings
        if let Some(precious) = self.try_place_precious_block(x, y, z, island) {
            return Some(precious);
        }

        // Stone layers with variation
        if depth_fraction > 0.7 && total_depth > 30 {
            // Deep: deepslate
            return Some(blocks::DEEPSLATE);
        }

        // Main body: stone with random granite/diorite/andesite
        let variant_hash = block_hash(x, y, z, self.seed);
        Some(match variant_hash % 20 {
            0 => blocks::GRANITE,
            1 => blocks::DIORITE,
            2 => blocks::ANDESITE,
            _ => blocks::STONE,
        })
    }

    /// Get the surface block for a biome.
    fn surface_block(&self, island: &IslandDef) -> BlockStateId {
        match island.biome {
            0 | 1 | 2 | 4 | 12 | 14 | 18 => blocks::GRASS_BLOCK,  // grass biomes
            3 | 10 => blocks::SAND,                                   // desert, badlands
            5 => blocks::GRASS_BLOCK,                                 // savanna
            6 => blocks::GRASS_BLOCK,                                 // taiga (podzol later)
            7 => blocks::SNOW_BLOCK,                                  // snowy
            8 => blocks::GRASS_BLOCK,                                 // swamp
            9 => blocks::GRASS_BLOCK,                                 // jungle
            11 => blocks::MYCELIUM,                                   // mushroom
            13 => blocks::STONE,                                      // mountain
            15 => blocks::NETHERRACK,                                 // volcanic
            16 => blocks::SNOW_BLOCK,                                 // cloud
            17 => blocks::COBBLESTONE,                                // floating ruin
            _ => blocks::GRASS_BLOCK,
        }
    }

    /// Get the topsoil block (below surface) for a biome.
    fn topsoil_block(&self, island: &IslandDef) -> BlockStateId {
        match island.biome {
            3 | 10 => blocks::SANDSTONE,   // desert/badlands
            7 | 16 => blocks::SNOW_BLOCK,  // snowy/cloud
            15 => blocks::NETHERRACK,      // volcanic
            17 => blocks::STONE,           // ruin
            _ => blocks::DIRT,
        }
    }

    /// Try to place an ore block based on depth and ring.
    fn try_place_ore(&self, x: i32, y: i32, z: i32, depth_frac: f64, island: &IslandDef) -> Option<BlockStateId> {
        let ring = island.ring;
        let h = block_hash(x, y, z, self.seed.wrapping_add(42));
        let chance = (h % 1000) as f64 / 1000.0;

        // Ring multiplier for ore density
        let ring_mult = 1.0 + ring as f64 * 0.1;

        // Coal: depth 0-100%, common
        if depth_frac > 0.05 && chance < 0.02 * ring_mult {
            return Some(blocks::COAL_ORE);
        }

        // Iron: depth 20-80%
        if depth_frac > 0.2 && depth_frac < 0.8 && chance < 0.015 * ring_mult {
            return Some(blocks::IRON_ORE);
        }

        // Copper: depth 10-60%
        if depth_frac > 0.1 && depth_frac < 0.6 && chance < 0.01 * ring_mult {
            return Some(blocks::COPPER_ORE);
        }

        // Lapis: ring 1+, depth 40-80%
        if ring >= 1 && depth_frac > 0.4 && depth_frac < 0.8 && chance < 0.005 * ring_mult {
            return Some(blocks::LAPIS_ORE);
        }

        // Gold: ring 1+, depth 50-90%
        if ring >= 1 && depth_frac > 0.5 && depth_frac < 0.9 && chance < 0.005 * ring_mult {
            return Some(blocks::GOLD_ORE);
        }

        // Redstone: ring 2+, depth 60-100%
        if ring >= 2 && depth_frac > 0.6 && chance < 0.006 * ring_mult {
            return Some(blocks::REDSTONE_ORE);
        }

        // Diamond: ring 2+, depth 80-100%
        if ring >= 2 && depth_frac > 0.8 && chance < 0.002 * ring_mult {
            return Some(blocks::DIAMOND_ORE);
        }

        // Emerald: ring 2+, mountain biome only, single blocks
        if ring >= 2 && island.biome == 13 && depth_frac > 0.5 && chance < 0.003 {
            return Some(blocks::EMERALD_ORE);
        }

        None
    }

    /// Generate a flat world chunk: one layer of GRASS_BLOCK at Y=64.
    fn generate_flat_chunk(&self, chunk_pos: ChunkPos) -> ChunkSection {
        let base_x = chunk_pos.x * 16;
        let base_y = chunk_pos.y * 16;
        let base_z = chunk_pos.z * 16;
        let mut section = ChunkSection::empty();

        // Grass floor at Y=64
        if base_y <= 64 && 64 < base_y + 16 {
            let ly = (64 - base_y) as u8;
            for lz in 0..16u8 {
                for lx in 0..16u8 {
                    set_block_in_section(&mut section, lx, ly, lz, blocks::GRASS_BLOCK);
                }
            }
        }

        // Place tree blocks from nearby chunk columns (radius 2)
        for tcx in (chunk_pos.x - 2)..=(chunk_pos.x + 2) {
            for tcz in (chunk_pos.z - 2)..=(chunk_pos.z + 2) {
                for tree in self.flat_trees_in_column(tcx, tcz) {
                    // Place each block of this tree if it falls in our chunk section
                    for (wx, wy, wz, block) in &tree {
                        let lx = wx - base_x;
                        let ly = wy - base_y;
                        let lz = wz - base_z;
                        if lx >= 0 && lx < 16 && ly >= 0 && ly < 16 && lz >= 0 && lz < 16 {
                            set_block_in_section(&mut section, lx as u8, ly as u8, lz as u8, *block);
                        }
                    }
                }
            }
        }

        section
    }

    /// Generate all trees for a chunk column (cx, cz). Returns world-space blocks.
    fn flat_trees_in_column(&self, cx: i32, cz: i32) -> Vec<TreeBlocks> {
        // Check cache first
        if let Ok(cache) = self.tree_cache.lock() {
            if let Some(cached) = cache.get(&(cx, cz)) {
                return cached.clone();
            }
        }

        let base_x = cx * 16;
        let base_z = cz * 16;
        let col_hash = block_hash(cx, 0, cz, self.seed);
        let tree_count = (col_hash % 3) as i32;
        let mut trees = Vec::new();

        for i in 0..tree_count {
            let th = block_hash(cx, i, cz, self.seed.wrapping_add(1));
            let tx = base_x + 2 + (th % 12) as i32;
            let tz = base_z + 2 + ((th >> 8) % 12) as i32;
            let tree_type = block_hash(tx, i as i32, tz, self.seed.wrapping_add(77)) % 8;
            trees.push(self.gen_tree(tx, tz, th, tree_type));
        }

        // Store in cache, evict if too large (keep last 512 columns)
        if let Ok(mut cache) = self.tree_cache.lock() {
            if cache.len() > 512 {
                cache.clear();
            }
            cache.insert((cx, cz), trees.clone());
        }

        trees
    }

    /// Generate a single tree at world pos (tx, 65, tz) by type.
    fn gen_tree(&self, tx: i32, tz: i32, th: u64, tree_type: u64) -> Vec<(i32, i32, i32, BlockStateId)> {
        let mut out: Vec<(i32, i32, i32, BlockStateId)> = Vec::new();
        let base_y = 65i32;

        let (log_id, leaf_id) = match tree_type {
            0 => (blocks::OAK_LOG,      blocks::OAK_LEAVES),
            1 => (blocks::SPRUCE_LOG,   blocks::SPRUCE_LEAVES),
            2 => (blocks::BIRCH_LOG,    blocks::BIRCH_LEAVES),
            3 => (blocks::JUNGLE_LOG,   blocks::JUNGLE_LEAVES),
            4 => (blocks::ACACIA_LOG,   blocks::ACACIA_LEAVES),
            5 => (blocks::CHERRY_LOG,   blocks::CHERRY_LEAVES),
            6 => (blocks::DARK_OAK_LOG, blocks::DARK_OAK_LEAVES),
            _ => (blocks::MANGROVE_LOG, blocks::MANGROVE_LEAVES),
        };

        match tree_type {
            // ── Oak: tall straight, round crown 5x5→3x3→1
            0 => {
                let h = 6 + (th >> 16) as i32 % 3; // 6-8
                for y in base_y..base_y + h {
                    out.push((tx, y, tz, log_id));
                }
                gen_round_crown(&mut out, tx, tz, base_y + h, leaf_id, &[3, 3, 3, 2, 1, 0], self.seed);
            }
            // ── Spruce: tall narrow, layered cone crown
            1 => {
                let h = 8 + (th >> 16) as i32 % 4; // 8-11
                for y in base_y..base_y + h {
                    out.push((tx, y, tz, log_id));
                }
                let crown_bottom = base_y + 1;
                let mut r = 4i32;
                let mut y = crown_bottom;
                while y <= base_y + h + 1 {
                    for dz in -r..=r {
                        for dx in -r..=r {
                            if dx.abs() + dz.abs() <= r + 1 {
                                if !(dx == 0 && dz == 0) {
                                    out.push((tx + dx, y, tz + dz, leaf_id));
                                }
                            }
                        }
                    }
                    y += 2;
                    r = (r - 1).max(0);
                }
                out.push((tx, base_y + h + 1, tz, leaf_id));
            }
            // ── Birch: medium straight, tall narrow oval crown
            2 => {
                let h = 5 + (th >> 16) as i32 % 3; // 5-7
                for y in base_y..base_y + h {
                    out.push((tx, y, tz, log_id));
                }
                gen_round_crown(&mut out, tx, tz, base_y + h, leaf_id, &[2, 3, 2, 1, 0], self.seed);
            }
            // ── Jungle: very tall, bushy crown + leaf clusters on trunk
            3 => {
                let h = 10 + (th >> 16) as i32 % 5; // 10-14
                for y in base_y..base_y + h {
                    out.push((tx, y, tz, log_id));
                }
                gen_round_crown(&mut out, tx, tz, base_y + h, leaf_id, &[3, 3, 3, 2, 1, 0], self.seed);
                // Leaf clusters on trunk at 1/3 and 2/3 height
                for frac in &[3, 2] {
                    let mid = base_y + h / frac;
                    for &(dx, dz) in &[(1i32,0),(-1,0),(0,1i32),(0,-1),(2,0),(-2,0),(0,2),(0,-2)] {
                        let lh = block_hash(tx+dx, mid, tz+dz, self.seed.wrapping_add(10));
                        if lh % 3 != 0 {
                            out.push((tx+dx, mid,   tz+dz, leaf_id));
                            out.push((tx+dx, mid-1, tz+dz, leaf_id));
                        }
                    }
                }
            }
            // ── Acacia: short, strongly leaning, wide flat umbrella
            4 => {
                let h = 5 + (th >> 16) as i32 % 2; // 5-6
                let mut sx = tx;
                let mut sz = tz;
                let lean_dir = (th >> 24) % 4;
                for y in base_y..base_y + h {
                    out.push((sx, y, sz, log_id));
                    if y >= base_y + h / 2 {
                        match lean_dir {
                            0 => sx += 1,
                            1 => sx -= 1,
                            2 => sz += 1,
                            _ => sz -= 1,
                        }
                    }
                }
                let cx = sx; let cz = sz;
                let top = base_y + h;
                // Wide flat umbrella: 7x7 bottom, 5x5 middle, 3x3 top
                for &(r, dy) in &[(3i32,0i32),(3,1),(2,2),(1,3)] {
                    for dz in -r..=r {
                        for dx in -r..=r {
                            if dx.abs() == r && dz.abs() == r {
                                let ch = block_hash(cx+dx, top+dy, cz+dz, self.seed.wrapping_add(4));
                                if ch % 3 == 0 { continue; }
                            }
                            out.push((cx+dx, top+dy, cz+dz, leaf_id));
                        }
                    }
                }
            }
            // ── Cherry: medium, massive fluffy cloud crown (many blobs)
            5 => {
                let h = 5 + (th >> 16) as i32 % 3; // 5-7
                for y in base_y..base_y + h {
                    out.push((tx, y, tz, log_id));
                }
                let top = base_y + h;
                // Dense blobs: 8 blobs, each 3x3x2
                for blob in 0..8i32 {
                    let bh = block_hash(tx, blob, tz, self.seed.wrapping_add(20 + blob as i64));
                    let bx = tx + (bh % 5) as i32 - 2;
                    let bz = tz + ((bh >> 4) % 5) as i32 - 2;
                    let by = top - 1 + (bh >> 8) as i32 % 3;
                    for dz in -1i32..=1 {
                        for dx in -1i32..=1 {
                            for dy in 0..2i32 {
                                out.push((bx+dx, by+dy, bz+dz, leaf_id));
                            }
                        }
                    }
                }
            }
            // ── Dark Oak: short wide, 2x2 trunk, massive layered crown
            6 => {
                let h = 5 + (th >> 16) as i32 % 2; // 5-6
                for y in base_y..base_y + h {
                    out.push((tx,   y, tz,   log_id));
                    out.push((tx+1, y, tz,   log_id));
                    out.push((tx,   y, tz+1, log_id));
                    out.push((tx+1, y, tz+1, log_id));
                }
                let top = base_y + h;
                for &(r, dy) in &[(4i32,0i32),(4,1),(3,2),(2,3),(1,4)] {
                    for dz in -r..=r {
                        for dx in -r..=r {
                            if dx.abs() == r && dz.abs() == r {
                                let ch = block_hash(tx+dx, top+dy, tz+dz, self.seed.wrapping_add(30));
                                if ch % 3 == 0 { continue; }
                            }
                            out.push((tx+dx, top+dy, tz+dz, leaf_id));
                        }
                    }
                }
            }
            // ── Mangrove: medium, stilt roots, bushy crown
            _ => {
                let h = 5 + (th >> 16) as i32 % 3; // 5-7
                for &(dx, dz) in &[(1i32,0i32),(0,1),(-1,0),(1,1)] {
                    let rh = block_hash(tx+dx, 0, tz+dz, self.seed.wrapping_add(40));
                    if rh % 2 == 0 {
                        out.push((tx+dx, base_y-1, tz+dz, log_id));
                        out.push((tx+dx, base_y,   tz+dz, log_id));
                    }
                }
                for y in base_y..base_y + h {
                    out.push((tx, y, tz, log_id));
                }
                gen_round_crown(&mut out, tx, tz, base_y + h, leaf_id, &[3, 3, 2, 1, 0], self.seed);
            }
        }

        out
    }

    /// Try to place precious blocks (gold/diamond/emerald blocks) at high rings.
    fn try_place_precious_block(&self, x: i32, y: i32, z: i32, island: &IslandDef) -> Option<BlockStateId> {
        let ring = island.ring;
        if ring < 20 {
            return None;
        }

        let h = block_hash(x, y, z, self.seed.wrapping_add(999));
        let chance = (h % 10000) as f64 / 10000.0;
        let precious_chance = ((ring as f64 - 20.0) * 0.002).min(0.3);

        if chance >= precious_chance {
            return None;
        }

        // Which precious block
        let type_hash = h % 100;
        Some(if ring >= 100 {
            match type_hash {
                0..=40 => blocks::GOLD_BLOCK,
                41..=70 => blocks::IRON_BLOCK,
                71..=90 => blocks::DIAMOND_BLOCK,
                _ => blocks::EMERALD_BLOCK,
            }
        } else {
            match type_hash {
                0..=60 => blocks::IRON_BLOCK,
                61..=85 => blocks::GOLD_BLOCK,
                86..=97 => blocks::DIAMOND_BLOCK,
                _ => blocks::EMERALD_BLOCK,
            }
        })
    }
}

/// Quick check if an island possibly overlaps a chunk (AABB test).
fn island_overlaps_chunk(island: &IslandDef, base_x: i32, base_y: i32, base_z: i32) -> bool {
    let chunk_max_x = base_x + 15;
    let chunk_max_y = base_y + 15;
    let chunk_max_z = base_z + 15;

    let island_min_x = island.center_x - island.radius_x;
    let island_max_x = island.center_x + island.radius_x;
    let island_min_z = island.center_z - island.radius_z;
    let island_max_z = island.center_z + island.radius_z;
    let island_min_y = island.base_y - island.depth;
    let island_max_y = island.base_y + 20; // surface variation buffer

    base_x <= island_max_x && chunk_max_x >= island_min_x
        && base_z <= island_max_z && chunk_max_z >= island_min_z
        && base_y <= island_max_y && chunk_max_y >= island_min_y
}

/// Set a block in a section, expanding from uniform if needed.
fn set_block_in_section(section: &mut ChunkSection, lx: u8, ly: u8, lz: u8, state: BlockStateId) {
    let index = (ly as usize) * 256 + (lz as usize) * 16 + (lx as usize);

    if section.blocks.is_empty() {
        let current = section.palette[0];
        if current == state {
            return;
        }
        section.blocks = vec![0; ChunkSection::VOLUME];
    }

    if let Some(palette_idx) = section.palette.iter().position(|&s| s == state) {
        section.blocks[index] = palette_idx as u16;
    } else {
        let new_idx = section.palette.len() as u16;
        section.palette.push(state);
        section.blocks[index] = new_idx;
    }
}


/// Generate a round crown. `radii` = radius per layer from bottom, 0 = single block.
fn gen_round_crown(
    out: &mut Vec<(i32, i32, i32, BlockStateId)>,
    cx: i32, cz: i32, top_y: i32,
    leaf_id: BlockStateId,
    radii: &[i32],
    seed: i64,
) {
    for (layer, &r) in radii.iter().enumerate() {
        let y = top_y + layer as i32 - radii.len() as i32 + 1;
        for dz in -r..=r {
            for dx in -r..=r {
                if dx.abs() == r && dz.abs() == r && r > 0 {
                    let h = block_hash(cx+dx, y, cz+dz, seed.wrapping_add(50));
                    if h % 2 == 0 { continue; }
                }
                let is_trunk = dx == 0 && dz == 0;
                if !is_trunk {
                    out.push((cx+dx, y, cz+dz, leaf_id));
                }
            }
        }
    }
    out.push((cx, top_y, cz, leaf_id));
}


/// Deterministic hash for a block position + seed. Fast, not cryptographic.
fn block_hash(x: i32, y: i32, z: i32, seed: i64) -> u64 {
    let mut h = seed as u64;
    h = h.wrapping_mul(6364136223846793005).wrapping_add(x as u64);
    h = h.wrapping_mul(6364136223846793005).wrapping_add(y as u64);
    h = h.wrapping_mul(6364136223846793005).wrapping_add(z as u64);
    h ^ (h >> 33)
}
