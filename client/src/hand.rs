// First-person hand rendering. Shows player's right arm in bottom-right of screen.
// Uses Steve skin texture, rendered in screen-space with bobbing and swing animation.

use glam::{Mat4, Vec3, Quat};
use crate::mesh::BlockVertex;

/// Steve skin texture size.
const SKIN_W: f32 = 64.0;
const SKIN_H: f32 = 64.0;

/// Right arm UV regions in steve.png (64x64, MC skin format).
/// Arm is 4x12x4 pixels. UV layout:
///   Top:    (44,16)-(48,20)
///   Bottom: (48,16)-(52,20)
///   Front:  (44,20)-(48,32)   (inner side, visible from 1st person)
///   Back:   (52,20)-(56,32)
///   Right:  (40,20)-(44,32)   (outer side)
///   Left:   (48,20)-(52,32)
struct ArmUV {
    top: [f32; 4],    // u0, v0, u1, v1
    bottom: [f32; 4],
    front: [f32; 4],
    back: [f32; 4],
    right: [f32; 4],
    left: [f32; 4],
}

const ARM_UV: ArmUV = ArmUV {
    top:    [44.0/SKIN_W, 16.0/SKIN_H, 48.0/SKIN_W, 20.0/SKIN_H],
    bottom: [48.0/SKIN_W, 16.0/SKIN_H, 52.0/SKIN_W, 20.0/SKIN_H],
    front:  [44.0/SKIN_W, 20.0/SKIN_H, 48.0/SKIN_W, 32.0/SKIN_H],
    back:   [52.0/SKIN_W, 20.0/SKIN_H, 56.0/SKIN_W, 32.0/SKIN_H],
    right:  [40.0/SKIN_W, 20.0/SKIN_H, 44.0/SKIN_W, 32.0/SKIN_H],
    left:   [48.0/SKIN_W, 20.0/SKIN_H, 52.0/SKIN_W, 32.0/SKIN_H],
};

/// Hand state for animation.
pub struct Hand {
    /// Swing animation progress (0.0 = rest, 1.0 = full swing). Resets after swing.
    pub swing_progress: f32,
    /// Whether a swing is in progress.
    pub is_swinging: bool,
    /// Walk cycle for bobbing (synced with camera).
    pub walk_cycle: f32,
    /// Whether walking (synced with camera).
    pub is_walking: bool,
}

impl Hand {
    pub fn new() -> Self {
        Self {
            swing_progress: 0.0,
            is_swinging: false,
            walk_cycle: 0.0,
            is_walking: false,
        }
    }

    /// Start a swing animation (on left click / attack).
    pub fn start_swing(&mut self) {
        self.is_swinging = true;
        self.swing_progress = 0.0;
    }

    /// Update animation each frame.
    pub fn update(&mut self, dt: f32) {
        // Swing animation
        if self.is_swinging {
            self.swing_progress += dt * 6.0; // ~0.17 sec full swing
            if self.swing_progress >= 1.0 {
                self.swing_progress = 0.0;
                self.is_swinging = false;
            }
        }
    }

    /// Build the hand model matrix (screen-space positioning).
    /// Returns a matrix that places the arm in bottom-right of view.
    pub fn model_matrix(&self) -> Mat4 {
        // Base position: right side, slightly below center, pushed forward
        let mut pos = Vec3::new(0.56, -0.52, -0.72);

        // Walk bobbing
        if self.is_walking {
            pos.y += self.walk_cycle.sin().abs() * 0.03;
            pos.x += (self.walk_cycle * 0.5).sin() * 0.02;
        }

        // Swing animation: rotate arm forward and slightly inward
        let mut rotation = Quat::IDENTITY;
        if self.is_swinging {
            // Smooth swing arc: fast start, slow end
            let t = self.swing_progress;
            let angle = swing_curve(t) * -1.5; // radians, forward swing
            let side_angle = swing_curve(t) * 0.3; // slight inward
            rotation = Quat::from_rotation_x(angle) * Quat::from_rotation_y(side_angle);
        }

        // Idle subtle sway
        let idle_sway = (self.walk_cycle * 0.3).sin() * 0.02;
        pos.x += idle_sway;

        // Scale: arm is thin and long
        let scale = Vec3::new(0.08, 0.24, 0.08);

        Mat4::from_translation(pos)
            * Mat4::from_quat(rotation)
            * Mat4::from_scale(scale)
    }

    /// Generate vertices and indices for the hand mesh.
    /// These are in local space -- multiply by model_matrix() for final position.
    /// Returns (vertices, indices).
    pub fn build_mesh(&self) -> (Vec<BlockVertex>, Vec<u32>) {
        let mut vertices = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);

        // Unit cube centered at origin, -0.5 to +0.5 on each axis
        // 6 faces, each with 4 vertices

        // Front face (+Z) -- this is what the player sees most
        add_face(&mut vertices, &mut indices,
            [[-0.5, -0.5,  0.5], [ 0.5, -0.5,  0.5], [ 0.5,  0.5,  0.5], [-0.5,  0.5,  0.5]],
            [0.0, 0.0, 1.0], &ARM_UV.front);

        // Back face (-Z)
        add_face(&mut vertices, &mut indices,
            [[ 0.5, -0.5, -0.5], [-0.5, -0.5, -0.5], [-0.5,  0.5, -0.5], [ 0.5,  0.5, -0.5]],
            [0.0, 0.0, -1.0], &ARM_UV.back);

        // Right face (+X) -- outer side of arm
        add_face(&mut vertices, &mut indices,
            [[ 0.5, -0.5,  0.5], [ 0.5, -0.5, -0.5], [ 0.5,  0.5, -0.5], [ 0.5,  0.5,  0.5]],
            [1.0, 0.0, 0.0], &ARM_UV.right);

        // Left face (-X) -- inner side
        add_face(&mut vertices, &mut indices,
            [[-0.5, -0.5, -0.5], [-0.5, -0.5,  0.5], [-0.5,  0.5,  0.5], [-0.5,  0.5, -0.5]],
            [-1.0, 0.0, 0.0], &ARM_UV.left);

        // Top face (+Y)
        add_face(&mut vertices, &mut indices,
            [[-0.5,  0.5,  0.5], [ 0.5,  0.5,  0.5], [ 0.5,  0.5, -0.5], [-0.5,  0.5, -0.5]],
            [0.0, 1.0, 0.0], &ARM_UV.top);

        // Bottom face (-Y)
        add_face(&mut vertices, &mut indices,
            [[-0.5, -0.5, -0.5], [ 0.5, -0.5, -0.5], [ 0.5, -0.5,  0.5], [-0.5, -0.5,  0.5]],
            [0.0, -1.0, 0.0], &ARM_UV.bottom);

        (vertices, indices)
    }
}

/// Add a quad face (4 vertices + 6 indices).
fn add_face(
    vertices: &mut Vec<BlockVertex>,
    indices: &mut Vec<u32>,
    positions: [[f32; 3]; 4],
    normal: [f32; 3],
    uv_region: &[f32; 4], // u0, v0, u1, v1
) {
    let base = vertices.len() as u32;

    let uvs = [
        [uv_region[0], uv_region[3]], // bottom-left
        [uv_region[2], uv_region[3]], // bottom-right
        [uv_region[2], uv_region[1]], // top-right
        [uv_region[0], uv_region[1]], // top-left
    ];

    for i in 0..4 {
        vertices.push(BlockVertex {
            position: positions[i],
            tex_coords: uvs[i],
            normal,
        });
    }

    indices.extend_from_slice(&[
        base, base + 1, base + 2,
        base, base + 2, base + 3,
    ]);
}

/// Smooth swing curve: fast at start, decelerates.
/// Input t: 0..1, output: 0..1 with easing.
fn swing_curve(t: f32) -> f32 {
    // Sine ease out
    (t * std::f32::consts::FRAC_PI_2).sin()
}

/// Hand shader: renders in screen-space, no fog, fixed lighting.
/// This is a separate WGSL shader for the hand overlay.
pub const HAND_SHADER_WGSL: &str = r#"
struct HandUniform {
    model: mat4x4<f32>,
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> hand: HandUniform;
@group(1) @binding(0) var t_skin: texture_2d<f32>;
@group(1) @binding(1) var s_skin: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) light: f32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_pos = hand.model * vec4<f32>(in.position, 1.0);
    out.clip_pos = hand.view_proj * world_pos;
    out.tex_coords = in.tex_coords;

    // Simple directional light for hand (always well-lit)
    let light_dir = normalize(vec3<f32>(0.3, 0.8, 0.5));
    let world_normal = normalize((hand.model * vec4<f32>(in.normal, 0.0)).xyz);
    let diffuse = max(dot(world_normal, light_dir), 0.0) * 0.5;
    out.light = 0.5 + diffuse; // 0.5 ambient + 0.5 directional max

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_skin, s_skin, in.tex_coords);

    if tex_color.a < 0.1 {
        discard;
    }

    let color = tex_color.rgb * in.light;
    return vec4<f32>(color, tex_color.a);
}
"#;

/// Hand uniform data for GPU. Must match shader struct.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HandUniform {
    pub model: [f32; 16],
    pub view_proj: [f32; 16],
}
