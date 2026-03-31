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
    state: AppState,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    input: InputState,
    world: ClientWorld,
    network: Option<NetworkClient>,
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
                // Init renderer (async, but we block here for simplicity)
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
                self.state = AppState::MainMenu;
            }
            Err(e) => {
                warn!("Failed to create window: {}", e);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Pass to input handler first
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
                // Main frame: update + render
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

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

impl App {
    /// Per-frame update logic.
    fn update(&mut self) {
        match self.state {
            AppState::MainMenu => {
                // TODO: draw main menu UI, handle connect button
            }
            AppState::Connecting => {
                // TODO: poll connection status
            }
            AppState::Playing => {
                // Process incoming packets from server
                if let Some(ref mut net) = self.network {
                    while let Some(packet) = net.try_recv() {
                        self.world.handle_server_packet(packet);
                    }
                }

                // Send player input to server
                // TODO: send position/look/actions
            }
            AppState::Initializing => {}
        }
    }
}
