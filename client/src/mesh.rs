// Chunk mesh generation with greedy meshing.
// Converts a 16x16x16 ChunkSection into GPU vertex/index buffers.

use skycraft_protocol::types::*;
use crate::atlas::{TextureAtlas, BlockFace, TileUV};

/// Vertex sent to GPU. Must match shader layout.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockVertex {
    /// World position.
    pub position: [f32; 3],
    /// Texture UV coordinates in atlas.
    pub tex_coords: [f32; 2],
    /// Face normal (for basic lighting).
    pub normal: [f32; 3],
}

impl BlockVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BlockVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // tex_coords
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // normal
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// Generated mesh data (CPU side, before upload to GPU).
pub struct ChunkMesh {
    pub vertices: Vec<BlockVertex>,
    pub indices: Vec<u32>,
    pub chunk_pos: ChunkPos,
}

impl ChunkMesh {
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

/// Axis direction for greedy meshing.
#[derive(Clone, Copy)]
struct FaceDir {
    /// Which axis is the face normal (0=X, 1=Y, 2=Z).
    axis: usize,
    /// +1 or -1 direction.
    sign: i32,
    /// BlockFace enum for atlas UV lookup.
    face: BlockFace,
    /// Normal vector.
    normal: [f32; 3],
}

const FACE_DIRS: [FaceDir; 6] = [
    FaceDir { axis: 1, sign:  1, face: BlockFace::Top,    normal: [ 0.0,  1.0,  0.0] },
    FaceDir { axis: 1, sign: -1, face: BlockFace::Bottom, normal: [ 0.0, -1.0,  0.0] },
    FaceDir { axis: 2, sign: -1, face: BlockFace::North,  normal: [ 0.0,  0.0, -1.0] },
    FaceDir { axis: 2, sign:  1, face: BlockFace::South,  normal: [ 0.0,  0.0,  1.0] },
    FaceDir { axis: 0, sign: -1, face: BlockFace::West,   normal: [-1.0,  0.0,  0.0] },
    FaceDir { axis: 0, sign:  1, face: BlockFace::East,   normal: [ 1.0,  0.0,  0.0] },
];

/// Build a mesh for a chunk section using greedy meshing.
///
/// `get_block`: returns block_state_id at world coordinates.
///   Must handle out-of-chunk lookups (for neighbor face culling).
pub fn build_chunk_mesh(
    chunk_pos: ChunkPos,
    section: &ChunkSection,
    atlas: &TextureAtlas,
    get_block: &dyn Fn(i32, i32, i32) -> BlockStateId,
) -> ChunkMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    if section.is_empty() {
        return ChunkMesh { vertices, indices, chunk_pos };
    }

    let base_x = chunk_pos.x * 16;
    let base_y = chunk_pos.y * 16;
    let base_z = chunk_pos.z * 16;

    for dir in &FACE_DIRS {
        greedy_face(
            dir,
            section,
            atlas,
            get_block,
            base_x, base_y, base_z,
            &mut vertices,
            &mut indices,
        );
    }

    ChunkMesh { vertices, indices, chunk_pos }
}

/// Greedy meshing for one face direction across the entire chunk section.
fn greedy_face(
    dir: &FaceDir,
    section: &ChunkSection,
    atlas: &TextureAtlas,
    get_block: &dyn Fn(i32, i32, i32) -> BlockStateId,
    base_x: i32,
    base_y: i32,
    base_z: i32,
    vertices: &mut Vec<BlockVertex>,
    indices: &mut Vec<u32>,
) {
    // The two axes perpendicular to the face normal.
    let (u_axis, v_axis) = match dir.axis {
        0 => (2, 1), // X normal -> iterate ZY
        1 => (0, 2), // Y normal -> iterate XZ
        2 => (0, 1), // Z normal -> iterate XY
        _ => unreachable!(),
    };

    // For each slice along the normal axis
    for d in 0..16i32 {
        // Build 16x16 mask of visible faces in this slice
        // Each cell: 0 = no face, >0 = block_state_id of the face
        let mut mask = [[0u16; 16]; 16];

        for v in 0..16i32 {
            for u in 0..16i32 {
                // Map (d, u, v) back to (x, y, z) in local coords
                let mut local = [0i32; 3];
                local[dir.axis] = d;
                local[u_axis] = u;
                local[v_axis] = v;

                let lx = local[0] as u8;
                let ly = local[1] as u8;
                let lz = local[2] as u8;

                let block = section.get_block(lx, ly, lz);

                // Skip air and non-textured blocks
                if block == 0 || !atlas.has_texture(block) {
                    continue;
                }

                // Check neighbor in face direction
                let mut neighbor_local = local;
                neighbor_local[dir.axis] += dir.sign;

                let neighbor = if neighbor_local[dir.axis] >= 0 && neighbor_local[dir.axis] < 16 {
                    // Neighbor is within this chunk
                    section.get_block(
                        neighbor_local[0] as u8,
                        neighbor_local[1] as u8,
                        neighbor_local[2] as u8,
                    )
                } else {
                    // Neighbor is in adjacent chunk - use world lookup
                    let wx = base_x + neighbor_local[0];
                    let wy = base_y + neighbor_local[1];
                    let wz = base_z + neighbor_local[2];
                    get_block(wx, wy, wz)
                };

                // Face is visible if neighbor is air or transparent
                if neighbor == 0 || atlas.is_transparent(neighbor) {
                    // Don't render face between two transparent blocks of same type
                    if atlas.is_transparent(block) && block == neighbor {
                        continue;
                    }
                    mask[v as usize][u as usize] = block;
                }
            }
        }

        // Greedy merge the mask into quads
        greedy_merge(
            &mut mask,
            dir,
            d,
            u_axis,
            v_axis,
            atlas,
            base_x, base_y, base_z,
            vertices,
            indices,
        );
    }
}

/// Greedy merge a 16x16 mask into as few quads as possible.
fn greedy_merge(
    mask: &mut [[u16; 16]; 16],
    dir: &FaceDir,
    d: i32,
    u_axis: usize,
    v_axis: usize,
    atlas: &TextureAtlas,
    base_x: i32,
    base_y: i32,
    base_z: i32,
    vertices: &mut Vec<BlockVertex>,
    indices: &mut Vec<u32>,
) {
    for v in 0..16 {
        let mut u = 0;
        while u < 16 {
            let block = mask[v][u];
            if block == 0 {
                u += 1;
                continue;
            }

            // Expand width (along u axis)
            let mut width = 1usize;
            while u + width < 16 && mask[v][u + width] == block {
                width += 1;
            }

            // Expand height (along v axis)
            let mut height = 1usize;
            'outer: while v + height < 16 {
                for wu in 0..width {
                    if mask[v + height][u + wu] != block {
                        break 'outer;
                    }
                }
                height += 1;
            }

            // Clear merged area from mask
            for hv in 0..height {
                for wu in 0..width {
                    mask[v + hv][u + wu] = 0;
                }
            }

            // Emit quad
            let uv = match atlas.get_uv(block, dir.face) {
                Some(uv) => uv,
                None => { u += width; continue; }
            };

            emit_quad(
                dir, d, u as i32, v as i32, width as i32, height as i32,
                u_axis, v_axis,
                base_x, base_y, base_z,
                &uv, width as f32, height as f32,
                vertices, indices,
            );

            u += width;
        }
    }
}

/// Emit a single quad (4 vertices + 6 indices) for a merged face.
fn emit_quad(
    dir: &FaceDir,
    d: i32,
    u_start: i32,
    v_start: i32,
    width: i32,
    height: i32,
    u_axis: usize,
    v_axis: usize,
    base_x: i32,
    base_y: i32,
    base_z: i32,
    uv: &TileUV,
    tex_scale_u: f32,
    tex_scale_v: f32,
    vertices: &mut Vec<BlockVertex>,
    indices: &mut Vec<u32>,
) {
    let base = [base_x as f32, base_y as f32, base_z as f32];

    // Face position along normal axis.
    // If sign=+1: face is on the far side of the block (d+1).
    // If sign=-1: face is on the near side (d).
    let face_d = if dir.sign > 0 { d as f32 + 1.0 } else { d as f32 };

    // Build 4 corner positions
    let mut corners = [[0.0f32; 3]; 4];
    for (i, corner) in corners.iter_mut().enumerate() {
        let cu = if i == 0 || i == 3 { u_start as f32 } else { (u_start + width) as f32 };
        let cv = if i == 0 || i == 1 { v_start as f32 } else { (v_start + height) as f32 };

        corner[dir.axis] = face_d + base[dir.axis];
        corner[u_axis] = cu + base[u_axis];
        corner[v_axis] = cv + base[v_axis];
    }

    // UV coordinates: tile the texture across the merged face
    let u0 = uv.u_min;
    let v0 = uv.v_min;
    let u1 = uv.u_min + (uv.u_max - uv.u_min) * tex_scale_u;
    let v1 = uv.v_min + (uv.v_max - uv.v_min) * tex_scale_v;

    let tex = [
        [u0, v0],
        [u1, v0],
        [u1, v1],
        [u0, v1],
    ];

    let base_idx = vertices.len() as u32;

    for i in 0..4 {
        vertices.push(BlockVertex {
            position: corners[i],
            tex_coords: tex[i],
            normal: dir.normal,
        });
    }

    // Two triangles. Winding order depends on face direction.
    if dir.sign > 0 {
        indices.extend_from_slice(&[
            base_idx, base_idx + 1, base_idx + 2,
            base_idx, base_idx + 2, base_idx + 3,
        ]);
    } else {
        indices.extend_from_slice(&[
            base_idx, base_idx + 2, base_idx + 1,
            base_idx, base_idx + 3, base_idx + 2,
        ]);
    }
}
