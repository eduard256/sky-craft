// Chunk meshing: converts a ChunkSection into vertex/index buffers for rendering.
// Checks adjacent blocks, only emits faces between solid and non-solid blocks.

use skycraft_protocol::types::{ChunkSection, ChunkPos, BlockStateId};
use super::vertex::Vertex;
use crate::world::ClientWorld;

/// Generated mesh data for a chunk section.
pub struct ChunkMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub chunk_pos: ChunkPos,
}

/// Build a mesh for a chunk section. Needs access to the world for neighbor checks.
pub fn build_chunk_mesh(world: &ClientWorld, chunk_pos: ChunkPos) -> Option<ChunkMesh> {
    let section = world.get_chunk(&chunk_pos)?;
    if section.is_empty() {
        return None;
    }

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let base_x = chunk_pos.x * 16;
    let base_y = chunk_pos.y * 16;
    let base_z = chunk_pos.z * 16;

    for ly in 0..16u8 {
        for lz in 0..16u8 {
            for lx in 0..16u8 {
                let block = section.get_block(lx, ly, lz);
                if block == 0 { continue; } // air

                let wx = base_x + lx as i32;
                let wy = base_y + ly as i32;
                let wz = base_z + lz as i32;

                let color = block_color(block);
                let x = lx as f32 + (base_x as f32);
                let y = ly as f32 + (base_y as f32);
                let z = lz as f32 + (base_z as f32);

                // Check each face: only emit if neighbor is air/transparent
                // +Y (top)
                if is_transparent_at(world, &section, lx, ly.wrapping_add(1), lz, wx, wy + 1, wz) {
                    add_face(&mut vertices, &mut indices, x, y, z, Face::Top, color);
                }
                // -Y (bottom)
                if is_transparent_at(world, &section, lx, ly.wrapping_sub(1), lz, wx, wy - 1, wz) {
                    add_face(&mut vertices, &mut indices, x, y, z, Face::Bottom, color);
                }
                // +X (east)
                if is_transparent_at(world, &section, lx.wrapping_add(1), ly, lz, wx + 1, wy, wz) {
                    add_face(&mut vertices, &mut indices, x, y, z, Face::East, color);
                }
                // -X (west)
                if is_transparent_at(world, &section, lx.wrapping_sub(1), ly, lz, wx - 1, wy, wz) {
                    add_face(&mut vertices, &mut indices, x, y, z, Face::West, color);
                }
                // +Z (south)
                if is_transparent_at(world, &section, lx, ly, lz.wrapping_add(1), wx, wy, wz + 1) {
                    add_face(&mut vertices, &mut indices, x, y, z, Face::South, color);
                }
                // -Z (north)
                if is_transparent_at(world, &section, lx, ly, lz.wrapping_sub(1), wx, wy, wz - 1) {
                    add_face(&mut vertices, &mut indices, x, y, z, Face::North, color);
                }
            }
        }
    }

    if vertices.is_empty() {
        return None;
    }

    Some(ChunkMesh { vertices, indices, chunk_pos })
}

/// Check if a neighbor position is transparent (should show face).
/// If local coords overflow (>15 or wraps to 255), look up in world.
fn is_transparent_at(
    world: &ClientWorld,
    section: &ChunkSection,
    lx: u8, ly: u8, lz: u8,
    wx: i32, wy: i32, wz: i32,
) -> bool {
    // If within same chunk section
    if lx < 16 && ly < 16 && lz < 16 {
        return section.get_block(lx, ly, lz) == 0;
    }
    // Neighbor chunk -- look up in world
    let block = world.get_block(skycraft_protocol::types::BlockPos::new(wx, wy, wz));
    block == 0
}

/// Map block state ID to a color for debug rendering.
fn block_color(block: BlockStateId) -> [f32; 3] {
    match block {
        1 => [0.5, 0.5, 0.5],        // stone - gray
        2 => [0.7, 0.5, 0.4],        // granite - pinkish
        4 => [0.8, 0.8, 0.8],        // diorite - light gray
        6 => [0.6, 0.6, 0.55],       // andesite - medium gray
        8 => [0.3, 0.7, 0.2],        // grass - green
        10 => [0.55, 0.35, 0.2],     // dirt - brown
        12 => [0.3, 0.3, 0.35],      // deepslate - dark gray
        14 => [0.45, 0.45, 0.45],    // cobblestone
        34 => [0.2, 0.3, 0.8],       // water - blue
        35 => [0.9, 0.4, 0.1],       // lava - orange
        36 => [0.15, 0.15, 0.15],    // coal ore - very dark
        37 => [0.7, 0.55, 0.45],     // iron ore - tan
        39 => [0.9, 0.8, 0.2],       // gold ore - yellow
        41 => [0.3, 0.9, 0.9],       // diamond ore - cyan
        43 => [0.2, 0.3, 0.8],       // lapis ore - blue
        45 => [0.7, 0.1, 0.1],       // redstone ore - red
        47 => [0.2, 0.8, 0.3],       // emerald ore - green
        66 => [0.9, 0.85, 0.6],      // sand - yellow
        67 => [0.85, 0.8, 0.55],     // sandstone
        155 => [0.95, 0.95, 0.95],   // snow - white
        248 => [0.5, 0.4, 0.5],      // mycelium - purple-gray
        249 => [0.5, 0.15, 0.15],    // netherrack - dark red
        255 => [1.0, 0.85, 0.0],     // gold block
        256 => [0.3, 0.95, 0.95],    // diamond block
        257 => [0.2, 0.9, 0.3],      // emerald block
        258 => [0.8, 0.8, 0.8],      // iron block
        _ => [0.6, 0.6, 0.6],        // unknown - medium gray
    }
}

enum Face { Top, Bottom, North, South, East, West }

/// Add a single block face (2 triangles) to the mesh.
fn add_face(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    x: f32, y: f32, z: f32,
    face: Face,
    color: [f32; 3],
) {
    let base = vertices.len() as u32;

    // Darken sides slightly for pseudo-lighting
    let (positions, normal, shade): ([[f32; 3]; 4], [f32; 3], f32) = match face {
        Face::Top => (
            [[x, y+1.0, z], [x+1.0, y+1.0, z], [x+1.0, y+1.0, z+1.0], [x, y+1.0, z+1.0]],
            [0.0, 1.0, 0.0], 1.0
        ),
        Face::Bottom => (
            [[x, y, z+1.0], [x+1.0, y, z+1.0], [x+1.0, y, z], [x, y, z]],
            [0.0, -1.0, 0.0], 0.5
        ),
        Face::North => (
            [[x+1.0, y, z], [x, y, z], [x, y+1.0, z], [x+1.0, y+1.0, z]],
            [0.0, 0.0, -1.0], 0.7
        ),
        Face::South => (
            [[x, y, z+1.0], [x+1.0, y, z+1.0], [x+1.0, y+1.0, z+1.0], [x, y+1.0, z+1.0]],
            [0.0, 0.0, 1.0], 0.7
        ),
        Face::East => (
            [[x+1.0, y, z+1.0], [x+1.0, y, z], [x+1.0, y+1.0, z], [x+1.0, y+1.0, z+1.0]],
            [1.0, 0.0, 0.0], 0.8
        ),
        Face::West => (
            [[x, y, z], [x, y, z+1.0], [x, y+1.0, z+1.0], [x, y+1.0, z]],
            [-1.0, 0.0, 0.0], 0.8
        ),
    };

    let shaded = [color[0] * shade, color[1] * shade, color[2] * shade];

    for pos in &positions {
        vertices.push(Vertex {
            position: *pos,
            color: shaded,
            normal,
        });
    }

    // Two triangles: 0-1-2, 0-2-3
    indices.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);
}
