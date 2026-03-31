// wgpu renderer. Handles GPU init, surface, and frame rendering with egui overlay.

use std::sync::Arc;
use wgpu::*;
use winit::window::Window;
use tracing::info;

use crate::state::AppScreen;

pub struct Renderer {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
}

impl Renderer {
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

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            size,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn device(&self) -> &Device { &self.device }
    pub fn queue(&self) -> &Queue { &self.queue }
    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> { self.size }
    pub fn surface_format(&self) -> TextureFormat { self.surface_config.format }

    pub fn create_encoder(&mut self) -> CommandEncoder {
        self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("egui_encoder"),
        })
    }

    /// Render frame with egui overlay.
    pub fn render_with_egui(
        &mut self,
        egui_renderer: &mut egui_wgpu::Renderer,
        paint_jobs: &[egui::ClippedPrimitive],
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        app_state: &AppScreen,
    ) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let clear_color = match app_state {
            AppScreen::MainMenu | AppScreen::Login | AppScreen::Connecting => {
                Color { r: 0.05, g: 0.05, b: 0.12, a: 1.0 }
            }
            AppScreen::Playing => {
                Color { r: 0.45, g: 0.65, b: 0.92, a: 1.0 }
            }
        };

        // Clear pass
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("clear_encoder"),
        });
        {
            let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("main_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(clear_color),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            // TODO: draw voxel world here
        }
        self.queue.submit(std::iter::once(encoder.finish()));

        // egui pass
        let mut encoder2 = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("egui_encoder"),
        });
        let render_pass = encoder2.begin_render_pass(&RenderPassDescriptor {
            label: Some("egui_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        // SAFETY: render_pass is dropped before encoder2.finish() via forget_lifetime
        let mut render_pass: wgpu::RenderPass<'static> = render_pass.forget_lifetime();
        egui_renderer.render(&mut render_pass, paint_jobs, screen_descriptor);
        drop(render_pass);
        self.queue.submit(std::iter::once(encoder2.finish()));

        output.present();

        Ok(())
    }
}
