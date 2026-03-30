// wgpu renderer with voxel mesh rendering pipeline.

pub mod vertex;
pub mod camera;
pub mod mesh;

use std::collections::HashMap;
use std::sync::Arc;
use wgpu::*;
use wgpu::util::DeviceExt;
use winit::window::Window;
use tracing::info;

use skycraft_protocol::types::ChunkPos;
use crate::world::ClientWorld;
use crate::input::InputState;
use crate::state::AppState;
use camera::{Camera, CameraUniform};
use vertex::Vertex;

/// GPU buffers for a single chunk mesh.
struct GpuChunkMesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
}

pub struct Renderer {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    // Render pipeline
    pipeline: RenderPipeline,
    depth_texture: Texture,
    depth_view: TextureView,

    // Camera
    pub camera: Camera,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,

    // Chunk meshes on GPU
    chunk_meshes: HashMap<ChunkPos, GpuChunkMesh>,
    meshes_dirty: bool,
}

impl Renderer {
    /// Initialize wgpu with the given window.
    pub async fn new(window: Arc<Window>) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();

        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("No suitable GPU adapter found")?;

        info!("GPU adapter: {}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("skycraft_device"),
                required_features: Features::empty(),
                required_limits: Limits::default(),
                ..Default::default()
            }, None)
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        // Camera uniform buffer
        let camera = Camera::new(size.width, size.height);
        let camera_uniform = camera.uniform();
        let camera_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),
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

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Shader
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("block_shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Depth texture
        let (depth_texture, depth_view) = create_depth_texture(&device, &surface_config);

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("block_pipeline_layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render pipeline
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("block_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            size,
            pipeline,
            depth_texture,
            depth_view,
            camera,
            camera_buffer,
            camera_bind_group,
            chunk_meshes: HashMap::new(),
            meshes_dirty: true,
        })
    }

    /// Handle window resize.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
            self.camera.resize(new_size.width, new_size.height);
            let (dt, dv) = create_depth_texture(&self.device, &self.surface_config);
            self.depth_texture = dt;
            self.depth_view = dv;
        }
    }

    /// Mark meshes as needing rebuild (call when chunks change).
    pub fn mark_dirty(&mut self) {
        self.meshes_dirty = true;
    }

    /// Rebuild GPU meshes from world chunk data.
    pub fn rebuild_meshes(&mut self, world: &ClientWorld) {
        self.chunk_meshes.clear();

        for chunk_pos in world.loaded_chunks() {
            if let Some(chunk_mesh) = mesh::build_chunk_mesh(world, *chunk_pos) {
                if chunk_mesh.vertices.is_empty() { continue; }

                let vertex_buffer = self.device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some("chunk_vb"),
                    contents: bytemuck::cast_slice(&chunk_mesh.vertices),
                    usage: BufferUsages::VERTEX,
                });
                let index_buffer = self.device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some("chunk_ib"),
                    contents: bytemuck::cast_slice(&chunk_mesh.indices),
                    usage: BufferUsages::INDEX,
                });

                self.chunk_meshes.insert(*chunk_pos, GpuChunkMesh {
                    vertex_buffer,
                    index_buffer,
                    index_count: chunk_mesh.indices.len() as u32,
                });
            }
        }

        self.meshes_dirty = false;
    }

    /// Render a frame.
    pub fn render(
        &mut self,
        world: &ClientWorld,
        _input: &InputState,
        _app_state: &AppState,
    ) -> Result<(), SurfaceError> {
        // Rebuild meshes if world changed
        if self.meshes_dirty {
            self.rebuild_meshes(world);
        }

        // Update camera uniform
        let camera_uniform = self.camera.uniform();
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("frame_encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("main_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color { r: 0.45, g: 0.65, b: 0.92, a: 1.0 }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // Draw all chunk meshes
            for mesh in self.chunk_meshes.values() {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

/// Create depth texture for z-buffering.
fn create_depth_texture(device: &Device, config: &SurfaceConfiguration) -> (Texture, TextureView) {
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("depth_texture"),
        size: Extent3d {
            width: config.width,
            height: config.height,
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
