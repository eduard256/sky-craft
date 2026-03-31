// Render pipeline: wgpu pipelines, bind groups, depth buffer, GPU buffers.
// Two pipelines: world blocks + hand overlay.

use std::collections::HashMap;
use wgpu::*;
use wgpu::util::DeviceExt;
use tracing::info;

use crate::atlas::TextureAtlas;
use crate::camera::CameraUniform;
use crate::hand::HandUniform;
use crate::mesh::{BlockVertex, ChunkMesh};
use skycraft_protocol::types::ChunkPos;

/// GPU buffers for a single chunk mesh.
pub struct ChunkGpuMesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
}

/// All render state: pipelines, bind groups, buffers.
pub struct RenderPipeline {
    // ── World pipeline ──
    pub world_pipeline: wgpu::RenderPipeline,
    pub camera_buffer: Buffer,
    pub camera_bind_group: BindGroup,
    pub atlas_bind_group: BindGroup,

    // ── Hand pipeline ──
    pub hand_pipeline: wgpu::RenderPipeline,
    pub hand_buffer: Buffer,
    pub hand_bind_group: BindGroup,
    pub skin_bind_group: BindGroup,
    pub hand_vertex_buffer: Buffer,
    pub hand_index_buffer: Buffer,
    pub hand_index_count: u32,

    // ── Depth ──
    pub depth_texture: Texture,
    pub depth_view: TextureView,

    // ── Chunk meshes on GPU ──
    pub chunk_meshes: HashMap<ChunkPos, ChunkGpuMesh>,
}

impl RenderPipeline {
    /// Create all pipelines and resources.
    pub fn new(
        device: &Device,
        queue: &Queue,
        surface_format: TextureFormat,
        width: u32,
        height: u32,
        atlas: &TextureAtlas,
        skin_texture_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // ── Depth texture ───────────────────────────────────────────────
        let (depth_texture, depth_view) = create_depth_texture(device, width, height);

        // ── Camera uniform ──────────────────────────────────────────────
        let camera_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("camera_uniform"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("camera_bgl"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("camera_bg"),
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // ── Atlas texture bind group ────────────────────────────────────
        let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("texture_bgl"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let atlas_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("atlas_bg"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&atlas.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&atlas.sampler),
                },
            ],
        });

        // ── World shader + pipeline ─────────────────────────────────────
        let world_shader_src = include_str!("shader.wgsl");
        let world_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("world_shader"),
            source: ShaderSource::Wgsl(world_shader_src.into()),
        });

        let world_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("world_pipeline_layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let world_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("world_pipeline"),
            layout: Some(&world_pipeline_layout),
            vertex: VertexState {
                module: &world_shader,
                entry_point: Some("vs_main"),
                buffers: &[BlockVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &world_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less, // world pipeline
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // ── Hand shader + pipeline ──────────────────────────────────────
        let hand_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("hand_shader"),
            source: ShaderSource::Wgsl(crate::hand::HAND_SHADER_WGSL.into()),
        });

        let hand_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("hand_uniform"),
            size: std::mem::size_of::<HandUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let hand_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("hand_bgl"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let hand_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("hand_bg"),
            layout: &hand_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: hand_buffer.as_entire_binding(),
            }],
        });

        // Load skin texture for hand
        let (skin_bind_group, _skin_texture) = load_skin_texture(
            device, queue, &texture_bind_group_layout, skin_texture_path,
        )?;

        let hand_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("hand_pipeline_layout"),
            bind_group_layouts: &[&hand_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let hand_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("hand_pipeline"),
            layout: Some(&hand_pipeline_layout),
            vertex: VertexState {
                module: &hand_shader,
                entry_point: Some("vs_main"),
                buffers: &[BlockVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &hand_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false, // hand always on top
                depth_compare: CompareFunction::Always,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Build hand mesh and upload to GPU
        let hand_model = crate::hand::Hand::new();
        let (hand_verts, hand_idxs) = hand_model.build_mesh();

        let hand_vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("hand_verts"),
            contents: bytemuck::cast_slice(&hand_verts),
            usage: BufferUsages::VERTEX,
        });
        let hand_index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("hand_idxs"),
            contents: bytemuck::cast_slice(&hand_idxs),
            usage: BufferUsages::INDEX,
        });

        info!("Render pipelines created (world + hand)");

        Ok(Self {
            world_pipeline,
            camera_buffer,
            camera_bind_group,
            atlas_bind_group,
            hand_pipeline,
            hand_buffer,
            hand_bind_group,
            skin_bind_group,
            hand_vertex_buffer,
            hand_index_buffer,
            hand_index_count: hand_idxs.len() as u32,
            depth_texture,
            depth_view,
            chunk_meshes: HashMap::new(),
        })
    }

    /// Upload a chunk mesh to GPU.
    pub fn remove_chunk_mesh(&mut self, pos: ChunkPos) {
        self.chunk_meshes.remove(&pos);
    }

    pub fn upload_chunk_mesh(&mut self, device: &Device, mesh: &ChunkMesh) {
        if mesh.is_empty() {
            self.chunk_meshes.remove(&mesh.chunk_pos);
            return;
        }

        let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("chunk_verts"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("chunk_idxs"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: BufferUsages::INDEX,
        });

        self.chunk_meshes.insert(mesh.chunk_pos, ChunkGpuMesh {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
        });
    }

    /// Recreate depth texture on resize.
    pub fn resize_depth(&mut self, device: &Device, width: u32, height: u32) {
        let (tex, view) = create_depth_texture(device, width, height);
        self.depth_texture = tex;
        self.depth_view = view;
    }

    /// Update camera uniform on GPU.
    pub fn update_camera(&self, queue: &Queue, uniform: &CameraUniform) {
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(uniform));
    }

    /// Update hand uniform on GPU.
    pub fn update_hand(&self, queue: &Queue, uniform: &HandUniform) {
        queue.write_buffer(&self.hand_buffer, 0, bytemuck::bytes_of(uniform));
    }
}

/// Create depth texture and view.
fn create_depth_texture(device: &Device, width: u32, height: u32) -> (Texture, TextureView) {
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("depth_texture"),
        size: Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&TextureViewDescriptor::default());
    (texture, view)
}

/// Load steve.png as GPU texture and create bind group.
fn load_skin_texture(
    device: &Device,
    queue: &Queue,
    layout: &BindGroupLayout,
    path: &str,
) -> Result<(BindGroup, Texture), Box<dyn std::error::Error>> {
    let img = image::open(path)?.to_rgba8();
    let (w, h) = img.dimensions();

    let size = Extent3d {
        width: w,
        height: h,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&TextureDescriptor {
        label: Some("skin_texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        &img,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * w),
            rows_per_image: Some(h),
        },
        size,
    );

    let view = texture.create_view(&TextureViewDescriptor::default());
    let sampler = device.create_sampler(&SamplerDescriptor {
        label: Some("skin_sampler"),
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        ..Default::default()
    });

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("skin_bg"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&sampler),
            },
        ],
    });

    Ok((bind_group, texture))
}
