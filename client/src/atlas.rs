// Texture Atlas: loads block textures, packs into single GPU texture,
// provides UV coordinate lookup by block_state_id + face.

use std::collections::HashMap;
use std::path::Path;
use image::{GenericImageView, RgbaImage};
use serde::Deserialize;
use tracing::{info, warn};
use wgpu;

use skycraft_protocol::types::BlockStateId;

/// Tile size in pixels (MC standard).
const TILE_SIZE: u32 = 16;

/// Atlas grid dimension (tiles per row/column). 32x32 = 1024 slots.
const ATLAS_GRID: u32 = 32;

/// Atlas image size in pixels.
const ATLAS_SIZE: u32 = ATLAS_GRID * TILE_SIZE; // 512x512

/// Which face of a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockFace {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

/// UV coordinates for a single tile in the atlas (0.0 - 1.0 range).
#[derive(Debug, Clone, Copy)]
pub struct TileUV {
    pub u_min: f32,
    pub v_min: f32,
    pub u_max: f32,
    pub v_max: f32,
}

/// Block texture info loaded from block_textures.json.
#[derive(Debug, Deserialize)]
struct BlockTextureEntry {
    name: String,
    transparent: Option<bool>,
    textures: Option<TextureDef>,
}

/// Texture definition from JSON.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TextureDef {
    AllFaces(AllFaces),
    PerFace(PerFace),
}

#[derive(Debug, Deserialize)]
struct AllFaces {
    all: String,
}

#[derive(Debug, Deserialize)]
struct PerFace {
    top: Option<String>,
    bottom: Option<String>,
    side: Option<String>,
    north: Option<String>,
    south: Option<String>,
    east: Option<String>,
    west: Option<String>,
}

/// Resolved textures for a block: 6 faces -> tile index in atlas.
struct BlockTextures {
    faces: [Option<u32>; 6], // index into atlas for [top, bottom, north, south, east, west]
    transparent: bool,
}

/// The texture atlas: one big texture + UV lookup per block+face.
pub struct TextureAtlas {
    /// GPU texture containing all tiles.
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,

    /// Block state ID -> per-face tile UV.
    /// Key: block_state_id. Value: [top, bottom, north, south, east, west] UVs.
    /// None entry = block has no texture (air, non-solid).
    block_uvs: HashMap<BlockStateId, Option<[TileUV; 6]>>,

    /// Whether block is transparent.
    transparent: HashMap<BlockStateId, bool>,
}

impl TextureAtlas {
    /// Build the atlas from block_textures.json and texture image files.
    pub fn build(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data_path: &str,
        textures_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Load block_textures.json
        let json_path = format!("{}/block_textures.json", data_path);
        let json_str = std::fs::read_to_string(&json_path)?;
        let entries: HashMap<String, BlockTextureEntry> = serde_json::from_str(&json_str)?;

        // Collect all unique texture filenames
        let mut unique_files: Vec<String> = Vec::new();
        let mut file_to_index: HashMap<String, u32> = HashMap::new();

        for entry in entries.values() {
            if let Some(ref tex_def) = entry.textures {
                let files = extract_filenames(tex_def);
                for f in files {
                    if !file_to_index.contains_key(&f) {
                        let idx = unique_files.len() as u32;
                        file_to_index.insert(f.clone(), idx);
                        unique_files.push(f);
                    }
                }
            }
        }

        info!("Atlas: {} unique textures to pack", unique_files.len());

        if unique_files.len() as u32 > ATLAS_GRID * ATLAS_GRID {
            warn!("Too many textures for atlas grid {}x{}", ATLAS_GRID, ATLAS_GRID);
        }

        // Create atlas image (RGBA)
        let mut atlas_image = RgbaImage::new(ATLAS_SIZE, ATLAS_SIZE);

        // Load each texture and blit into atlas
        for (idx, filename) in unique_files.iter().enumerate() {
            let file_path = format!("{}/{}", textures_path, filename);
            let tile_x = (idx as u32) % ATLAS_GRID;
            let tile_y = (idx as u32) / ATLAS_GRID;

            match image::open(&file_path) {
                Ok(img) => {
                    let img = img.to_rgba8();
                    // Only copy first 16x16 pixels (some textures are animated strips)
                    let copy_w = img.width().min(TILE_SIZE);
                    let copy_h = img.height().min(TILE_SIZE);

                    for py in 0..copy_h {
                        for px in 0..copy_w {
                            let pixel = img.get_pixel(px, py);
                            let dest_x = tile_x * TILE_SIZE + px;
                            let dest_y = tile_y * TILE_SIZE + py;
                            atlas_image.put_pixel(dest_x, dest_y, *pixel);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to load texture {}: {}", file_path, e);
                }
            }
        }

        info!("Atlas image built: {}x{} pixels", ATLAS_SIZE, ATLAS_SIZE);

        // Upload to GPU
        let texture_size = wgpu::Extent3d {
            width: ATLAS_SIZE,
            height: ATLAS_SIZE,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("block_atlas"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &atlas_image,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * ATLAS_SIZE),
                rows_per_image: Some(ATLAS_SIZE),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("block_atlas_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest, // pixelated look (MC style)
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Build UV lookup per block_state_id
        let mut block_uvs: HashMap<BlockStateId, Option<[TileUV; 6]>> = HashMap::new();
        let mut transparent: HashMap<BlockStateId, bool> = HashMap::new();

        for (state_id_str, entry) in &entries {
            let state_id: BlockStateId = match state_id_str.parse() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let is_transparent = entry.transparent.unwrap_or(false);
            transparent.insert(state_id, is_transparent);

            match &entry.textures {
                None => {
                    block_uvs.insert(state_id, None);
                }
                Some(tex_def) => {
                    let face_files = resolve_faces(tex_def);
                    let mut face_uvs = [TileUV { u_min: 0.0, v_min: 0.0, u_max: 0.0, v_max: 0.0 }; 6];

                    for (i, filename) in face_files.iter().enumerate() {
                        if let Some(&tile_idx) = file_to_index.get(filename.as_str()) {
                            face_uvs[i] = tile_uv(tile_idx);
                        }
                    }

                    block_uvs.insert(state_id, Some(face_uvs));
                }
            }
        }

        info!("Atlas: {} blocks mapped ({} with textures)",
            block_uvs.len(),
            block_uvs.values().filter(|v| v.is_some()).count(),
        );

        Ok(Self {
            texture,
            texture_view,
            sampler,
            block_uvs,
            transparent,
        })
    }

    /// Get UV coordinates for a block face. Returns None if block has no texture.
    pub fn get_uv(&self, state_id: BlockStateId, face: BlockFace) -> Option<TileUV> {
        let uvs = self.block_uvs.get(&state_id)?.as_ref()?;
        let idx = match face {
            BlockFace::Top => 0,
            BlockFace::Bottom => 1,
            BlockFace::North => 2,
            BlockFace::South => 3,
            BlockFace::East => 4,
            BlockFace::West => 5,
        };
        Some(uvs[idx])
    }

    /// Check if a block is transparent (glass, water, leaves, air, etc).
    pub fn is_transparent(&self, state_id: BlockStateId) -> bool {
        // state_id 0 = air, always transparent
        if state_id == 0 {
            return true;
        }
        self.transparent.get(&state_id).copied().unwrap_or(false)
    }

    /// Check if a block has any texture at all (renderable as cube).
    pub fn has_texture(&self, state_id: BlockStateId) -> bool {
        matches!(self.block_uvs.get(&state_id), Some(Some(_)))
    }
}

/// Calculate UV coordinates for a tile at given index in the atlas grid.
fn tile_uv(index: u32) -> TileUV {
    let tile_x = index % ATLAS_GRID;
    let tile_y = index / ATLAS_GRID;

    let u_min = tile_x as f32 / ATLAS_GRID as f32;
    let v_min = tile_y as f32 / ATLAS_GRID as f32;
    let u_max = (tile_x + 1) as f32 / ATLAS_GRID as f32;
    let v_max = (tile_y + 1) as f32 / ATLAS_GRID as f32;

    TileUV { u_min, v_min, u_max, v_max }
}

/// Extract all unique filenames from a texture definition.
fn extract_filenames(def: &TextureDef) -> Vec<String> {
    match def {
        TextureDef::AllFaces(a) => vec![a.all.clone()],
        TextureDef::PerFace(p) => {
            let mut files = Vec::new();
            for f in [&p.top, &p.bottom, &p.side, &p.north, &p.south, &p.east, &p.west] {
                if let Some(name) = f {
                    if !files.contains(name) {
                        files.push(name.clone());
                    }
                }
            }
            files
        }
    }
}

/// Resolve a texture definition into 6 face filenames [top, bottom, north, south, east, west].
fn resolve_faces(def: &TextureDef) -> [String; 6] {
    match def {
        TextureDef::AllFaces(a) => {
            let f = a.all.clone();
            [f.clone(), f.clone(), f.clone(), f.clone(), f.clone(), f]
        }
        TextureDef::PerFace(p) => {
            let side = p.side.clone().unwrap_or_default();
            [
                p.top.clone().unwrap_or_else(|| side.clone()),
                p.bottom.clone().unwrap_or_else(|| side.clone()),
                p.north.clone().unwrap_or_else(|| side.clone()),
                p.south.clone().unwrap_or_else(|| side.clone()),
                p.east.clone().unwrap_or_else(|| side.clone()),
                p.west.clone().unwrap_or(side),
            ]
        }
    }
}
