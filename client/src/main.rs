// Sky Craft Client - Entry point
// Thin client: connects to server, renders world, sends input.
// All game logic runs on the server.

mod network;
mod renderer;
mod world;
mod input;
mod ui;
mod audio;
mod state;
mod session;
mod auth_client;
pub mod atlas;
pub mod mesh;
pub mod camera;
pub mod hand;
pub mod pipeline;
pub mod net_bridge;
pub mod asset_downloader;
pub mod cow;

use tracing::info;
use winit::event_loop::EventLoop;

fn main() {
    // Init logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "skycraft_client=info,wgpu=warn".into()),
        )
        .init();

    info!("Sky Craft Client v{}", env!("CARGO_PKG_VERSION"));

    // Create winit event loop and window
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let mut app = state::App::new();

    info!("Starting event loop...");
    event_loop.run_app(&mut app).expect("Event loop error");
}
