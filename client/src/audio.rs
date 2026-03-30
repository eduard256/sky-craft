// Audio system. Loads and plays sounds using rodio.
// In v0.0.1: stub module. Full 3D audio is TODO.

/// Audio manager. Handles loading sound files and playing them.
pub struct AudioManager {
    /// Whether audio is enabled.
    pub enabled: bool,
    /// Master volume (0.0 - 1.0).
    pub master_volume: f32,
    /// Music volume (0.0 - 1.0).
    pub music_volume: f32,
    /// Sound effects volume (0.0 - 1.0).
    pub sfx_volume: f32,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            enabled: true,
            master_volume: 1.0,
            music_volume: 0.5,
            sfx_volume: 1.0,
        }
    }

    /// Play a sound effect at a world position (3D positional audio).
    pub fn play_sound(&self, _sound_id: u16, _x: f64, _y: f64, _z: f64, _volume: f32, _pitch: f32) {
        // TODO: load ogg from client/assets/sounds/, play with rodio
        // TODO: 3D audio: calculate panning and volume from listener position
    }

    /// Play background music track.
    pub fn play_music(&self, _track_name: &str) {
        // TODO: load and stream music file
    }

    /// Stop current music.
    pub fn stop_music(&self) {
        // TODO
    }

    /// Update listener position (call each frame with camera position).
    pub fn update_listener(&self, _x: f64, _y: f64, _z: f64, _yaw: f32) {
        // TODO: update 3D audio listener
    }
}
