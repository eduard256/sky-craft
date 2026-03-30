// Application state and winit event handling.
// Manages the main loop: window events -> input -> network -> render.

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId, WindowAttributes};
use tracing::{info, warn};

use crate::renderer::Renderer;
use crate::input::InputState;
use crate::world::ClientWorld;
use crate::network::NetworkClient;

/// Application state machine.
pub enum AppState {
    /// Waiting for window to be created.
    Initializing,
    /// Main menu (connect to server).
    MainMenu,
    /// Connecting to server.
    Connecting,
    /// In-game.
    Playing,
}

/// Top-level application struct.
pub struct App {
    pub state: AppState,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    input: InputState,
    world: ClientWorld,
    network: Option<NetworkClient>,
    /// Whether we've injected test chunks for offline preview.
    demo_loaded: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Initializing,
            window: None,
            renderer: None,
            input: InputState::new(),
            world: ClientWorld::new(),
            network: None,
            demo_loaded: false,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Creating window...");
        let attrs = WindowAttributes::default()
            .with_title("Sky Craft")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

        match event_loop.create_window(attrs) {
            Ok(window) => {
                let window = Arc::new(window);
                let renderer = pollster::block_on(Renderer::new(window.clone()));
                match renderer {
                    Ok(r) => {
                        info!("Renderer initialized");
                        self.renderer = Some(r);
                    }
                    Err(e) => {
                        warn!("Failed to init renderer: {}", e);
                    }
                }
                self.window = Some(window);
                // Start in Playing state with demo world for testing
                self.state = AppState::Playing;
            }
            Err(e) => {
                warn!("Failed to create window: {}", e);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        self.input.handle_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size);
                }
            }
            WindowEvent::RedrawRequested => {
                self.update();

                if let Some(renderer) = &mut self.renderer {
                    match renderer.render(&self.world, &self.input, &self.state) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            if let Some(window) = &self.window {
                                renderer.resize(window.inner_size());
                            }
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            warn!("Out of GPU memory");
                            event_loop.exit();
                        }
                        Err(e) => {
                            warn!("Render error: {:?}", e);
                        }
                    }
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

impl App {
    fn update(&mut self) {
        // Load demo world on first frame if no server connection
        if !self.demo_loaded && self.network.is_none() {
            self.load_demo_world();
            self.demo_loaded = true;
            if let Some(renderer) = &mut self.renderer {
                renderer.mark_dirty();
            }
        }

        // Process camera input
        if let Some(renderer) = &mut self.renderer {
            // Mouse look
            renderer.camera.process_mouse(self.input.mouse_dx, self.input.mouse_dy);

            // WASD movement (free-fly for now, no gravity)
            renderer.camera.process_movement(
                self.input.is_forward(),
                self.input.is_backward(),
                self.input.is_left(),
                self.input.is_right(),
                self.input.is_jump(),
                self.input.is_sneak(),
                self.input.is_sprint(),
            );
        }

        // Clear per-frame input state
        self.input.begin_frame();

        // Network
        match self.state {
            AppState::Playing => {
                if let Some(ref mut net) = self.network {
                    while let Some(packet) = net.try_recv() {
                        self.world.handle_server_packet(packet);
                        if let Some(renderer) = &mut self.renderer {
                            renderer.mark_dirty();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Load a small demo world for offline testing. Generates a flat island.
    fn load_demo_world(&mut self) {
        use skycraft_protocol::types::*;

        info!("Loading demo world...");

        // Generate a 3x3 chunk flat island at Y=64 (chunk Y=4)
        for cx in -2..=2 {
            for cz in -2..=2 {
                // Surface layer (grass) at chunk y=4, local y=0 (block y=64)
                let mut surface_section = ChunkSection::empty();
                for lz in 0..16u8 {
                    for lx in 0..16u8 {
                        // Grass on top
                        set_block(&mut surface_section, lx, 0, lz, 8); // grass
                    }
                }

                // Dirt/stone below surface at chunk y=3 (block y=48-63)
                let mut under_section = ChunkSection::empty();
                for ly in 0..16u8 {
                    for lz in 0..16u8 {
                        for lx in 0..16u8 {
                            let dist = ((lx as f32 - 8.0).powi(2) + (lz as f32 - 8.0).powi(2)).sqrt();
                            // Taper toward edges for floating island look
                            let max_depth = (16.0 - dist * 0.8).max(0.0) as u8;
                            if (15 - ly) < max_depth {
                                if ly > 12 {
                                    set_block(&mut under_section, lx, ly, lz, 10); // dirt
                                } else if ly > 4 {
                                    set_block(&mut under_section, lx, ly, lz, 1); // stone
                                    // Occasional ore
                                    if (lx.wrapping_mul(7).wrapping_add(ly.wrapping_mul(13)).wrapping_add(lz.wrapping_mul(19))) % 20 == 0 {
                                        set_block(&mut under_section, lx, ly, lz, 36); // coal
                                    }
                                    if (lx.wrapping_mul(11).wrapping_add(ly.wrapping_mul(17)).wrapping_add(lz.wrapping_mul(23))) % 30 == 0 {
                                        set_block(&mut under_section, lx, ly, lz, 37); // iron
                                    }
                                } else {
                                    // Deep stone with more ore
                                    set_block(&mut under_section, lx, ly, lz, 12); // deepslate
                                    if (lx.wrapping_add(ly).wrapping_add(lz)) % 25 == 0 {
                                        set_block(&mut under_section, lx, ly, lz, 41); // diamond
                                    }
                                }
                            }
                        }
                    }
                }

                self.world.insert_chunk(ChunkPos::new(cx, 4, cz), surface_section);
                self.world.insert_chunk(ChunkPos::new(cx, 3, cz), under_section);
            }
        }

        // Set camera above the island
        if let Some(renderer) = &mut self.renderer {
            renderer.camera.position = glam::Vec3::new(0.0, 70.0, -20.0);
            renderer.camera.pitch = -20.0;
        }

        info!("Demo world loaded: {} chunks", self.world.loaded_chunk_count());
    }
}

/// Helper to set a block in a section.
fn set_block(section: &mut skycraft_protocol::types::ChunkSection, lx: u8, ly: u8, lz: u8, state: skycraft_protocol::types::BlockStateId) {
    let index = (ly as usize) * 256 + (lz as usize) * 16 + (lx as usize);
    if section.blocks.is_empty() {
        let current = section.palette[0];
        if current == state { return; }
        section.blocks = vec![0; skycraft_protocol::types::ChunkSection::VOLUME];
    }
    if let Some(palette_idx) = section.palette.iter().position(|&s| s == state) {
        section.blocks[index] = palette_idx as u16;
    } else {
        let new_idx = section.palette.len() as u16;
        section.palette.push(state);
        section.blocks[index] = new_idx;
    }
}
