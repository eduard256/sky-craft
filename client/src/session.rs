// Session persistence: save/load auth token to ~/.skycraft/session.json

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub nickname: String,
    pub token: String,
}

/// Get the session file path: ~/.skycraft/session.json
fn session_path() -> PathBuf {
    let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join(".skycraft").join("session.json")
}

/// Load saved session, if any.
pub fn load_session() -> Option<Session> {
    let path = session_path();
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Save session to disk.
pub fn save_session(session: &Session) -> Result<(), Box<dyn std::error::Error>> {
    let path = session_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(session)?;
    std::fs::write(path, data)?;
    Ok(())
}

/// Delete saved session.
pub fn delete_session() {
    let path = session_path();
    let _ = std::fs::remove_file(path);
}
