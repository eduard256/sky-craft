// First-person camera. Handles view/projection matrices and mouse look.

use glam::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,   // degrees, 0 = +Z (south)
    pub pitch: f32,  // degrees, -89..+89
    pub fov: f32,    // degrees
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    pub sensitivity: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            position: Vec3::new(0.0, 80.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: 70.0,
            aspect: width as f32 / height.max(1) as f32,
            near: 0.1,
            far: 1000.0,
            sensitivity: 0.15,
        }
    }

    /// Update camera look from mouse delta.
    pub fn process_mouse(&mut self, dx: f64, dy: f64) {
        self.yaw += dx as f32 * self.sensitivity;
        self.pitch -= dy as f32 * self.sensitivity;
        self.pitch = self.pitch.clamp(-89.0, 89.0);
        self.yaw = self.yaw.rem_euclid(360.0);
    }

    /// Get the direction the camera is looking.
    pub fn forward(&self) -> Vec3 {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();
        Vec3::new(
            -yaw_rad.sin() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.cos() * pitch_rad.cos(),
        ).normalize()
    }

    /// Get the right vector (perpendicular to forward on XZ plane).
    pub fn right(&self) -> Vec3 {
        let yaw_rad = self.yaw.to_radians();
        Vec3::new(-yaw_rad.cos(), 0.0, -yaw_rad.sin()).normalize()
    }

    /// Move camera based on WASD input.
    pub fn process_movement(&mut self, forward: bool, backward: bool, left: bool, right: bool, up: bool, down: bool, sprint: bool) {
        let speed = if sprint { 0.3 } else { 0.15 };
        let fwd = self.forward();
        let rt = self.right();

        // Only move on XZ plane for forward/backward/strafe
        let fwd_xz = Vec3::new(fwd.x, 0.0, fwd.z).normalize_or_zero();

        if forward  { self.position += fwd_xz * speed; }
        if backward { self.position -= fwd_xz * speed; }
        if left     { self.position -= rt * speed; }
        if right    { self.position += rt * speed; }
        if up       { self.position.y += speed; }
        if down     { self.position.y -= speed; }
    }

    /// Update aspect ratio on window resize.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height.max(1) as f32;
    }

    /// Build the view-projection matrix.
    pub fn build_view_proj(&self) -> Mat4 {
        let view = Mat4::look_to_rh(self.position, self.forward(), Vec3::Y);
        let proj = Mat4::perspective_rh(
            self.fov.to_radians(),
            self.aspect,
            self.near,
            self.far,
        );
        proj * view
    }

    /// Get the uniform data for the GPU.
    pub fn uniform(&self) -> CameraUniform {
        CameraUniform {
            view_proj: self.build_view_proj().to_cols_array_2d(),
        }
    }
}
