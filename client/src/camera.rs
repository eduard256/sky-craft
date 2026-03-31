// First-person camera. Handles view/projection matrices, mouse look, movement vectors.
// MC-compatible: yaw 0-360, pitch -90..+90, FOV 70, eye height 1.62.

use glam::{Mat4, Vec3};

/// Player eye height above feet position (MC standard).
const EYE_HEIGHT: f32 = 1.62;

/// Default field of view in degrees.
const DEFAULT_FOV: f32 = 70.0;

/// Mouse sensitivity: degrees per pixel of mouse movement.
const DEFAULT_SENSITIVITY: f32 = 0.15;

/// Near clipping plane (close enough to see hand).
const NEAR_PLANE: f32 = 0.05;

/// Sprint FOV bonus in degrees.
const SPRINT_FOV_BONUS: f32 = 10.0;

/// FOV interpolation speed (per frame, 0-1).
const FOV_LERP_SPEED: f32 = 0.15;

pub struct Camera {
    /// Player feet position (from server).
    pub position: Vec3,

    /// Horizontal rotation in radians. 0 = +Z (south), increases clockwise.
    pub yaw: f32,

    /// Vertical rotation in radians. 0 = horizontal, negative = up, positive = down.
    pub pitch: f32,

    /// Base field of view in degrees.
    pub fov: f32,

    /// Current effective FOV (smoothly interpolates toward target).
    pub current_fov: f32,

    /// Mouse sensitivity (degrees per pixel).
    pub sensitivity: f32,

    /// Aspect ratio (width / height).
    pub aspect: f32,

    /// Far clipping plane in blocks.
    pub far_plane: f32,

    /// Whether player is sprinting (affects FOV).
    pub is_sprinting: bool,

    /// Walk cycle timer for bobbing (radians, increases while walking).
    pub walk_cycle: f32,

    /// Whether player is walking (for bobbing).
    pub is_walking: bool,
}

/// Camera uniform data to upload to GPU. Must match shader struct layout.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [f32; 16],
    pub camera_pos: [f32; 3],
    pub _pad0: f32,
    pub sun_dir: [f32; 3],
    pub _pad1: f32,
    pub fog_color: [f32; 3],
    pub fog_start: f32,
    pub fog_end: f32,
    pub time_of_day: f32,
    pub _pad2: f32,
    pub _pad3: f32,
}

impl Camera {
    pub fn new(render_distance_chunks: u8) -> Self {
        Self {
            position: Vec3::new(0.0, 80.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: DEFAULT_FOV,
            current_fov: DEFAULT_FOV,
            sensitivity: DEFAULT_SENSITIVITY,
            aspect: 16.0 / 9.0,
            far_plane: render_distance_chunks as f32 * 16.0,
            is_sprinting: false,
            walk_cycle: 0.0,
            is_walking: false,
        }
    }

    /// Process raw mouse delta (in pixels). Updates yaw and pitch.
    pub fn process_mouse(&mut self, dx: f64, dy: f64) {
        let dx = dx as f32 * self.sensitivity;
        let dy = dy as f32 * self.sensitivity;

        // Yaw: mouse right = rotate right (decrease yaw)
        self.yaw -= dx.to_radians();

        // Wrap yaw to 0..2PI
        self.yaw = self.yaw.rem_euclid(std::f32::consts::TAU);

        // Pitch: mouse up = look up (decrease pitch, MC convention)
        self.pitch -= dy.to_radians();

        // Clamp pitch to just under +-90 degrees
        let max_pitch = 89.9_f32.to_radians();
        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);
    }

    /// Update aspect ratio on window resize.
    pub fn set_aspect(&mut self, width: u32, height: u32) {
        if height > 0 {
            self.aspect = width as f32 / height as f32;
        }
    }

    /// Per-frame update: FOV smoothing, walk bobbing.
    pub fn update(&mut self, dt: f32) {
        // Smooth FOV transition (sprint)
        let target_fov = if self.is_sprinting {
            self.fov + SPRINT_FOV_BONUS
        } else {
            self.fov
        };
        self.current_fov += (target_fov - self.current_fov) * FOV_LERP_SPEED;

        // Walk bobbing cycle
        if self.is_walking {
            self.walk_cycle += dt * 8.0; // ~8 rad/sec = comfortable walk bob
        } else {
            // Smoothly return to neutral
            self.walk_cycle = 0.0;
        }
    }

    /// Eye position (feet position + eye height + bobbing offset).
    pub fn eye_position(&self) -> Vec3 {
        let mut eye = self.position + Vec3::new(0.0, EYE_HEIGHT, 0.0);

        // Walk bobbing
        if self.is_walking {
            let bob_y = self.walk_cycle.sin().abs() * 0.06;
            let bob_x = (self.walk_cycle * 0.5).sin() * 0.03;
            eye.y += bob_y;
            eye.x += bob_x * self.yaw.cos();
            eye.z += bob_x * self.yaw.sin();
        }

        eye
    }

    /// Forward direction vector from yaw and pitch.
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.cos(),
        ).normalize()
    }

    /// Forward direction on XZ plane only (for walking, ignores pitch).
    pub fn forward_xz(&self) -> Vec3 {
        Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos()).normalize()
    }

    /// Right direction on XZ plane (for strafing).
    pub fn right_xz(&self) -> Vec3 {
        let fwd = self.forward_xz();
        Vec3::new(fwd.z, 0.0, -fwd.x)
    }

    /// View matrix (world -> camera space).
    pub fn view_matrix(&self) -> Mat4 {
        let eye = self.eye_position();
        let target = eye + self.forward();
        Mat4::look_at_rh(eye, target, Vec3::Y)
    }

    /// Projection matrix (camera -> clip space).
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(
            self.current_fov.to_radians(),
            self.aspect,
            NEAR_PLANE,
            self.far_plane,
        )
    }

    /// Combined view-projection matrix.
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Build GPU uniform data.
    pub fn build_uniform(&self, time_of_day_normalized: f32) -> CameraUniform {
        let vp = self.view_projection_matrix();
        let eye = self.eye_position();

        // Sun direction based on time of day
        // time 0.0 = midnight (sun below), 0.5 = noon (sun above)
        let sun_angle = (time_of_day_normalized - 0.25) * std::f32::consts::TAU;
        let sun_dir = Vec3::new(
            0.3, // slight east offset
            sun_angle.sin(),
            sun_angle.cos() * 0.5,
        ).normalize();

        // Fog color: lerp between day sky and night dark
        let day_factor = (sun_dir.y.max(0.0) * 2.0).min(1.0);
        let day_fog = Vec3::new(0.55, 0.72, 0.92);   // light blue
        let night_fog = Vec3::new(0.02, 0.02, 0.05);  // near black
        let fog = day_fog * day_factor + night_fog * (1.0 - day_factor);

        CameraUniform {
            view_proj: vp.to_cols_array(),
            camera_pos: eye.to_array(),
            _pad0: 0.0,
            sun_dir: sun_dir.to_array(),
            _pad1: 0.0,
            fog_color: fog.to_array(),
            fog_start: self.far_plane * 0.6,
            fog_end: self.far_plane * 0.95,
            time_of_day: time_of_day_normalized,
            _pad2: 0.0,
            _pad3: 0.0,
        }
    }

    /// Calculate movement vector from input state.
    /// Returns desired velocity direction (not magnitude).
    pub fn movement_vector(&self, forward: bool, backward: bool, left: bool, right: bool) -> Vec3 {
        let mut dir = Vec3::ZERO;

        if forward { dir += self.forward_xz(); }
        if backward { dir -= self.forward_xz(); }
        if left { dir -= self.right_xz(); }
        if right { dir += self.right_xz(); }

        if dir.length_squared() > 0.001 {
            dir.normalize()
        } else {
            Vec3::ZERO
        }
    }
}
