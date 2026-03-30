// Chunk storage. Thread-safe concurrent hashmap of chunk sections.

use dashmap::DashMap;
use skycraft_protocol::types::*;

/// Thread-safe chunk storage backed by DashMap.
pub struct ChunkStore {
    sections: DashMap<ChunkPos, ChunkSection>,
}

impl ChunkStore {
    pub fn new() -> Self {
        Self {
            sections: DashMap::new(),
        }
    }

    /// Get a chunk section by position.
    pub fn get(&self, pos: &ChunkPos) -> Option<ChunkSection> {
        self.sections.get(pos).map(|s| s.value().clone())
    }

    /// Insert or replace a chunk section.
    pub fn insert(&self, pos: ChunkPos, section: ChunkSection) {
        self.sections.insert(pos, section);
    }

    /// Set a single block within a chunk. Creates the chunk if needed.
    pub fn set_block(&self, chunk_pos: ChunkPos, lx: u8, ly: u8, lz: u8, state: BlockStateId) {
        let mut entry = self.sections.entry(chunk_pos).or_insert_with(ChunkSection::empty);
        let section = entry.value_mut();

        let index = (ly as usize) * 256 + (lz as usize) * 16 + (lx as usize);

        // If section is uniform (single palette entry), expand it
        if section.blocks.is_empty() {
            let current = section.palette[0];
            if current == state {
                return; // already the same block
            }
            // Expand: fill blocks array with index 0 (current block)
            section.blocks = vec![0; ChunkSection::VOLUME];
        }

        // Check if state is already in palette
        if let Some(palette_idx) = section.palette.iter().position(|&s| s == state) {
            section.blocks[index] = palette_idx as u16;
        } else {
            // Add new state to palette
            let new_idx = section.palette.len() as u16;
            section.palette.push(state);
            section.blocks[index] = new_idx;
        }
    }

    /// Number of loaded chunks.
    pub fn len(&self) -> usize {
        self.sections.len()
    }

    /// Check if a chunk is loaded.
    pub fn contains(&self, pos: &ChunkPos) -> bool {
        self.sections.contains_key(pos)
    }

    /// Remove a chunk from memory.
    pub fn unload(&self, pos: &ChunkPos) {
        self.sections.remove(pos);
    }
}
