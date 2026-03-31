// World: chunk storage, generation, block access.

pub mod chunk;
pub mod generator;
pub mod island;

use std::sync::Arc;
use skycraft_protocol::types::*;

use crate::config::ServerConfig;
use chunk::ChunkStore;
use generator::WorldGenerator;

/// The game world. Thread-safe, stores chunks and provides block access.
pub struct World {
    pub seed: i64,
    chunks: ChunkStore,
    generator: WorldGenerator,
    config: Arc<ServerConfig>,
}

impl World {
    pub fn new(seed: i64, config: Arc<ServerConfig>) -> Self {
        let mut generator = WorldGenerator::new(seed);
        generator.flat = config.flat_world;
        Self {
            seed,
            chunks: ChunkStore::new(),
            generator,
            config,
        }
    }

    /// Get a chunk section, generating it if not yet loaded.
    pub fn get_or_generate_chunk(&self, pos: ChunkPos) -> ChunkSection {
        if let Some(section) = self.chunks.get(&pos) {
            return section;
        }

        // Generate the chunk
        let section = self.generator.generate_chunk(pos);
        self.chunks.insert(pos, section.clone());
        section
    }

    /// Get block state at a world position.
    pub fn get_block(&self, pos: BlockPos) -> BlockStateId {
        let chunk_pos = pos.to_chunk_pos();
        let (lx, ly, lz) = pos.chunk_local();
        let section = self.get_or_generate_chunk(chunk_pos);
        section.get_block(lx, ly, lz)
    }

    /// Set a block at a world position. Returns the old block state.
    pub fn set_block(&self, pos: BlockPos, state: BlockStateId) -> BlockStateId {
        let chunk_pos = pos.to_chunk_pos();
        let (lx, ly, lz) = pos.chunk_local();

        // Ensure chunk exists
        let section = self.get_or_generate_chunk(chunk_pos);
        let old = section.get_block(lx, ly, lz);

        // Modify the chunk
        self.chunks.set_block(chunk_pos, lx, ly, lz, state);

        old
    }

    /// Get the world spawn position (center of ring 0, on a suitable island).
    pub fn get_spawn_position(&self) -> EntityPos {
        if self.config.flat_world {
            return EntityPos::new(0.5, 65.0, 0.5);
        }
        // Find the first solid block near origin
        // Search in a spiral from 0,0 outward for a valid spawn point
        for radius in 0i32..100 {
            for x in -radius..=radius {
                for z in -radius..=radius {
                    if x.abs() != radius && z.abs() != radius {
                        continue; // only check perimeter of current radius
                    }
                    // Check column for solid ground
                    for y in (40..120).rev() {
                        let pos = BlockPos::new(x, y, z);
                        let block = self.get_block(pos);
                        let above = self.get_block(BlockPos::new(x, y + 1, z));
                        let above2 = self.get_block(BlockPos::new(x, y + 2, z));

                        // Solid block with 2 air blocks above
                        if block != 0 && above == 0 && above2 == 0 {
                            return EntityPos::new(
                                x as f64 + 0.5,
                                y as f64 + 1.0,
                                z as f64 + 0.5,
                            );
                        }
                    }
                }
            }
        }

        // Fallback
        EntityPos::new(0.5, 80.0, 0.5)
    }

    /// Check if a block position is within any island (not void).
    pub fn is_on_island(&self, pos: BlockPos) -> bool {
        self.get_block(pos) != 0
    }

    /// Calculate ring number for a world position.
    pub fn ring_at(&self, x: f64, z: f64) -> RingNumber {
        let dist = (x * x + z * z).sqrt();
        (dist / 500.0) as RingNumber
    }
}
