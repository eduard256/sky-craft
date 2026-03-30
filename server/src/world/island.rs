// Island shape and placement logic. Determines where islands spawn and their geometry.

use rand::prelude::*;
use rand::SeedableRng;

/// Metadata for a generated island.
#[derive(Debug, Clone)]
pub struct IslandDef {
    /// Center X in block coords.
    pub center_x: i32,
    /// Center Z in block coords.
    pub center_z: i32,
    /// Base Y level (top surface average height).
    pub base_y: i32,
    /// Approximate radius in X direction (half-width).
    pub radius_x: i32,
    /// Approximate radius in Z direction (half-length).
    pub radius_z: i32,
    /// Maximum depth below base_y.
    pub depth: i32,
    /// Biome index.
    pub biome: u8,
    /// Ring this island belongs to.
    pub ring: u32,
    /// Island name seed (for procedural naming).
    pub name_seed: u64,
    /// Whether this island has a village.
    pub has_village: bool,
    /// Whether this island has a loot chest.
    pub has_loot_chest: bool,
}

/// Grid cell size for island placement (blocks).
const CELL_SIZE: i32 = 400;

/// Determine which islands exist near a given chunk position.
/// Returns all islands whose bounding area intersects a region around the chunk.
pub fn islands_near_chunk(chunk_x: i32, chunk_z: i32, seed: i64) -> Vec<IslandDef> {
    let block_x = chunk_x * 16;
    let block_z = chunk_z * 16;

    // Check the cell this chunk is in plus all 8 neighbors
    let cell_x = block_x.div_euclid(CELL_SIZE);
    let cell_z = block_z.div_euclid(CELL_SIZE);

    let mut islands = Vec::new();
    for dx in -1..=1 {
        for dz in -1..=1 {
            let cx = cell_x + dx;
            let cz = cell_z + dz;
            islands_in_cell(cx, cz, seed, &mut islands);
        }
    }
    islands
}

/// Generate island definitions for a single grid cell.
fn islands_in_cell(cell_x: i32, cell_z: i32, seed: i64, out: &mut Vec<IslandDef>) {
    let cell_seed = hash_cell(cell_x, cell_z, seed);
    let mut rng = StdRng::seed_from_u64(cell_seed);

    // Distance from world origin determines ring
    let world_x = cell_x * CELL_SIZE + CELL_SIZE / 2;
    let world_z = cell_z * CELL_SIZE + CELL_SIZE / 2;
    let dist = ((world_x as f64).powi(2) + (world_z as f64).powi(2)).sqrt();
    let ring = (dist / 500.0) as u32;

    // Number of islands in this cell (1-3, higher near origin)
    let island_count = if ring == 0 {
        rng.random_range(2..=3)
    } else if ring < 5 {
        rng.random_range(1..=3)
    } else {
        rng.random_range(0..=2)
    };

    for i in 0..island_count {
        // Position within cell (with padding to avoid edge overlap)
        let padding = 40;
        let cx = cell_x * CELL_SIZE + rng.random_range(padding..(CELL_SIZE - padding));
        let cz = cell_z * CELL_SIZE + rng.random_range(padding..(CELL_SIZE - padding));

        // Size scales with ring slightly, but mostly random
        let base_radius = match ring {
            0 => rng.random_range(50..200),
            1..=3 => rng.random_range(30..300),
            4..=10 => rng.random_range(20..250),
            _ => rng.random_range(10..200),
        };
        let radius_x = base_radius + rng.random_range(-20..20i32).max(5);
        let radius_z = base_radius + rng.random_range(-20..20i32).max(5);

        // Depth: roughly proportional to radius
        let depth = (base_radius as f32 * rng.random_range(0.3..0.8)) as i32;
        let depth = depth.clamp(10, 150);

        // Base Y: increases with ring
        let base_y = 64 + (ring as i32 * 3).min(256) + rng.random_range(-10..10);

        // Biome: depends on ring and randomness
        let biome = pick_biome(ring, &mut rng);

        // Village: only on large plains/desert/savanna/taiga islands
        let has_village = base_radius >= 150
            && matches!(biome, 0 | 3 | 5 | 6) // plains, desert, savanna, taiga
            && rng.random_range(0.0..1.0f32) < 0.3;

        // Every island has a loot chest
        let has_loot_chest = true;

        let name_seed = rng.random();

        out.push(IslandDef {
            center_x: cx,
            center_z: cz,
            base_y,
            radius_x,
            radius_z,
            depth,
            biome,
            ring,
            name_seed,
            has_village,
            has_loot_chest,
        });
    }
}

/// Pick a biome index for an island based on ring.
/// 0=plains, 1=forest, 2=birch_forest, 3=desert, 4=dark_forest,
/// 5=savanna, 6=taiga, 7=snowy, 8=swamp, 9=jungle, 10=badlands,
/// 11=mushroom, 12=flower_forest, 13=mountain, 14=meadow,
/// 15=volcanic (ring 3+), 16=cloud (ring 5+), 17=floating_ruin (ring 4+),
/// 18=garden (ring 2+)
fn pick_biome(ring: u32, rng: &mut StdRng) -> u8 {
    let mut options: Vec<u8> = vec![0, 1, 2, 12, 14]; // always available

    if ring >= 1 {
        options.extend_from_slice(&[4, 6, 8]);
    }
    if ring >= 2 {
        options.extend_from_slice(&[3, 5, 9, 10, 13, 18]);
    }
    if ring >= 3 {
        options.extend_from_slice(&[7, 11, 15, 17]);
    }
    if ring >= 5 {
        options.push(16);
    }

    // Mushroom and garden are rare
    let idx = rng.random_range(0..options.len());
    options[idx]
}

/// Hash a cell coordinate pair with seed to get a deterministic cell seed.
fn hash_cell(x: i32, z: i32, seed: i64) -> u64 {
    let mut h = seed as u64;
    h = h.wrapping_mul(6364136223846793005).wrapping_add(x as u64);
    h = h.wrapping_mul(6364136223846793005).wrapping_add(z as u64);
    h ^ (h >> 33)
}

/// Get the name for an island based on its name seed and biome.
pub fn island_name(name_seed: u64, biome: u8) -> String {
    let adjectives = [
        "Verdant", "Barren", "Windy", "Misty", "Ancient", "Forgotten",
        "Frozen", "Burning", "Silent", "Hollow", "Crimson", "Golden",
        "Shadowed", "Bright", "Rugged", "Serene", "Wild", "Lonely",
    ];
    let suffixes = [
        "Isle", "Crest", "Rock", "Peak", "Haven", "Reef",
        "Spire", "Shelf", "Drift", "Perch", "Bluff", "Ledge",
    ];
    let biome_name = biome_display_name(biome);

    let adj_idx = (name_seed % adjectives.len() as u64) as usize;
    let suf_idx = ((name_seed >> 16) % suffixes.len() as u64) as usize;

    format!("{} {} {}", adjectives[adj_idx], biome_name, suffixes[suf_idx])
}

/// Human-readable biome name.
pub fn biome_display_name(biome: u8) -> &'static str {
    match biome {
        0 => "Plains",
        1 => "Forest",
        2 => "Birch Forest",
        3 => "Desert",
        4 => "Dark Forest",
        5 => "Savanna",
        6 => "Taiga",
        7 => "Snowy Plains",
        8 => "Swamp",
        9 => "Jungle",
        10 => "Badlands",
        11 => "Mushroom",
        12 => "Flower Forest",
        13 => "Mountain",
        14 => "Meadow",
        15 => "Volcanic",
        16 => "Cloud",
        17 => "Floating Ruin",
        18 => "Garden",
        _ => "Unknown",
    }
}

impl IslandDef {
    /// Check if a block position is within this island's horizontal bounding area.
    pub fn contains_xz(&self, x: i32, z: i32) -> bool {
        let dx = (x - self.center_x) as f64 / self.radius_x as f64;
        let dz = (z - self.center_z) as f64 / self.radius_z as f64;
        (dx * dx + dz * dz) <= 1.0 // ellipse check
    }

    /// Calculate the top surface height at a given XZ position.
    /// Returns None if position is outside the island.
    pub fn surface_y(&self, x: i32, z: i32, seed: i64) -> Option<i32> {
        if !self.contains_xz(x, z) {
            return None;
        }

        let dx = (x - self.center_x) as f64 / self.radius_x as f64;
        let dz = (z - self.center_z) as f64 / self.radius_z as f64;
        let dist_norm = (dx * dx + dz * dz).sqrt(); // 0 at center, 1 at edge

        // Edge falloff: terrain drops near edges
        let edge_factor = 1.0 - dist_norm.powi(3);

        // Simple height noise (deterministic from position + seed)
        let noise_val = simple_noise(x, z, seed);

        // Biome-specific height variation
        let variation = match self.biome {
            0 | 14 => 4.0,   // plains/meadow: flat
            1 | 2 | 12 => 6.0, // forests: gentle
            3 => 2.0,        // desert: very flat
            4 => 5.0,        // dark forest
            5 => 3.0,        // savanna
            6 => 8.0,        // taiga: hilly
            7 => 5.0,        // snowy
            8 => 2.0,        // swamp: flat
            9 => 10.0,       // jungle: hilly
            10 => 6.0,       // badlands
            13 => 15.0,      // mountain: very hilly
            15 => 8.0,       // volcanic
            _ => 4.0,
        };

        let height = self.base_y as f64 + noise_val * variation * edge_factor;
        Some(height as i32)
    }

    /// Calculate the bottom surface height at a given XZ position.
    /// Returns None if outside island.
    pub fn bottom_y(&self, x: i32, z: i32, seed: i64) -> Option<i32> {
        let surface = self.surface_y(x, z, seed)?;

        let dx = (x - self.center_x) as f64 / self.radius_x as f64;
        let dz = (z - self.center_z) as f64 / self.radius_z as f64;
        let dist_norm = (dx * dx + dz * dz).sqrt();

        // Taper: depth reduces toward edges (Avatar-style pointed bottom)
        let taper = (1.0 - dist_norm.powf(0.7)).max(0.05);
        let depth = self.depth as f64 * taper;

        // Add noise to bottom surface for organic shape
        let noise_val = simple_noise(x + 1000, z + 1000, seed);
        let bottom = surface as f64 - depth + noise_val * depth * 0.2;

        Some(bottom as i32)
    }

    /// Get the display name for this island.
    pub fn name(&self) -> String {
        island_name(self.name_seed, self.biome)
    }
}

/// Simple deterministic noise function (not Perlin, just hash-based).
/// Returns value in -1.0 to 1.0 range.
fn simple_noise(x: i32, z: i32, seed: i64) -> f64 {
    let h = hash_cell(x, z, seed);
    // Map to -1.0 .. 1.0
    (h as f64 / u64::MAX as f64) * 2.0 - 1.0
}
