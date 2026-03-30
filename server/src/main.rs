// Sky Craft Server - Entry point
// Authoritative game server. All game logic runs here.
// Client is a dumb renderer that only draws what server tells it.

mod config;
mod network;
mod world;
mod game;
mod player;
mod entity;
mod physics;
mod auth;

use std::sync::Arc;
use tracing::{info, error};

#[tokio::main]
async fn main() {
    // Init logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "skycraft_server=info".into()),
        )
        .init();

    info!("Sky Craft Server v{}", env!("CARGO_PKG_VERSION"));

    // Load config
    let config = match config::ServerConfig::load() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };
    info!("Config loaded: port={}, max_players={}, seed={}", config.port, config.max_players, config.seed);

    let config = Arc::new(config);

    // Init world
    let world = Arc::new(world::World::new(config.seed, config.clone()));
    info!("World initialized with seed {}", config.seed);

    // Init game state
    let game_state = Arc::new(game::GameState::new(config.clone(), world.clone()));

    // Start game loop in background
    let game_loop_state = game_state.clone();
    tokio::spawn(async move {
        game::game_loop(game_loop_state).await;
    });
    info!("Game loop started at {} TPS", skycraft_protocol::TICKS_PER_SECOND);

    // Start network listener (blocks)
    if let Err(e) = network::start_server(config, game_state).await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}
