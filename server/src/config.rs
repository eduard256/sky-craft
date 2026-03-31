// Server configuration loaded from server.toml or defaults.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    // ── Network ──
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_max_players")]
    pub max_players: u32,
    #[serde(default = "default_view_distance")]
    pub view_distance: u8,
    #[serde(default = "default_motd")]
    pub motd: String,

    // ── World ──
    #[serde(default = "default_seed")]
    pub seed: i64,
    #[serde(default = "default_world_name")]
    pub world_name: String,
    #[serde(default = "default_spawn_protection")]
    pub spawn_protection: u32,

    // ── Gameplay ──
    #[serde(default = "default_difficulty")]
    pub difficulty: String,
    #[serde(default = "default_true")]
    pub pvp: bool,
    #[serde(default)]
    pub keep_inventory: bool,
    #[serde(default = "default_true")]
    pub mob_griefing: bool,
    #[serde(default = "default_true")]
    pub fire_tick: bool,
    #[serde(default = "default_true")]
    pub natural_regeneration: bool,
    #[serde(default = "default_true")]
    pub do_mob_spawning: bool,
    #[serde(default = "default_true")]
    pub do_daylight_cycle: bool,
    #[serde(default = "default_true")]
    pub do_weather_cycle: bool,
    #[serde(default = "default_sleeping_percentage")]
    pub players_sleeping_percentage: u8,

    // ── Debug ──
    #[serde(default)]
    pub flat_world: bool,

    // ── Auth ──
    #[serde(default = "default_auth_url")]
    pub auth_api_url: String,

    // ── Paths ──
    #[serde(default = "default_world_dir")]
    pub world_dir: String,
}

fn default_port() -> u16 { skycraft_protocol::DEFAULT_PORT }
fn default_max_players() -> u32 { 50 }
fn default_view_distance() -> u8 { skycraft_protocol::DEFAULT_VIEW_DISTANCE }
fn default_motd() -> String { "Sky Craft Server".to_string() }
fn default_seed() -> i64 { rand::random() }
fn default_world_name() -> String { "world".to_string() }
fn default_spawn_protection() -> u32 { 16 }
fn default_difficulty() -> String { "normal".to_string() }
fn default_true() -> bool { true }
fn default_sleeping_percentage() -> u8 { 50 }
fn default_auth_url() -> String { "https://apiskycraft.webaweba.com".to_string() }
fn default_world_dir() -> String { "./world_data".to_string() }

impl ServerConfig {
    /// Load config from server.toml if it exists, otherwise use defaults.
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new("server.toml");
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: ServerConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = ServerConfig::default();
            // Write default config so admin can edit it
            let toml_str = toml::to_string_pretty(&config)?;
            std::fs::write(path, toml_str)?;
            Ok(config)
        }
    }

    pub fn difficulty_enum(&self) -> skycraft_protocol::types::Difficulty {
        match self.difficulty.to_lowercase().as_str() {
            "peaceful" => skycraft_protocol::types::Difficulty::Peaceful,
            "easy" => skycraft_protocol::types::Difficulty::Easy,
            "hard" => skycraft_protocol::types::Difficulty::Hard,
            _ => skycraft_protocol::types::Difficulty::Normal,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            max_players: default_max_players(),
            view_distance: default_view_distance(),
            motd: default_motd(),
            seed: default_seed(),
            world_name: default_world_name(),
            spawn_protection: default_spawn_protection(),
            difficulty: default_difficulty(),
            pvp: true,
            keep_inventory: false,
            mob_griefing: true,
            fire_tick: true,
            natural_regeneration: true,
            do_mob_spawning: true,
            do_daylight_cycle: true,
            do_weather_cycle: true,
            players_sleeping_percentage: default_sleeping_percentage(),
            auth_api_url: default_auth_url(),
            world_dir: default_world_dir(),
            flat_world: false,
        }
    }
}
