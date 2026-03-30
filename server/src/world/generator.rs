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
    pub const OAK_LOG: BlockStateId = 73;
    pub const OAK_LEAVES: BlockStateId = 144;
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
    pub const BIRCH_LOG: BlockStateId = 78;
    pub const BIRCH_LEAVES: BlockStateId = 148;
    pub const SPRUCE_LOG: BlockStateId = 75;
    pub const SPRUCE_LEAVES: BlockStateId = 146;
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

/// World generator produces chunk sections from seed + position.
pub struct WorldGenerator {
    seed: i64,
}

impl WorldGenerator {
    pub fn new(seed: i64) -> Self {
        Self { seed }
    }

    /// Generate a chunk section at the given chunk position.
    pub fn generate_chunk(&self, chunk_pos: ChunkPos) -> ChunkSection {
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

/// Deterministic hash for a block position + seed. Fast, not cryptographic.
fn block_hash(x: i32, y: i32, z: i32, seed: i64) -> u64 {
    let mut h = seed as u64;
    h = h.wrapping_mul(6364136223846793005).wrapping_add(x as u64);
    h = h.wrapping_mul(6364136223846793005).wrapping_add(y as u64);
    h = h.wrapping_mul(6364136223846793005).wrapping_add(z as u64);
    h ^ (h >> 33)
}
