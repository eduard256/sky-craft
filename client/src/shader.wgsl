// Sky Craft block shader.
// Single-pass: vertex transform + textured fragment with lighting.
// Optimized: all lighting computed per-vertex (not per-pixel), interpolated.

// ── Uniforms ────────────────────────────────────────────────────────────────

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _pad0: f32,
    // Sun direction (normalized, points toward sun)
    sun_dir: vec3<f32>,
    _pad1: f32,
    // Fog
    fog_color: vec3<f32>,
    fog_start: f32,
    fog_end: f32,
    // Time 0-1 (0=midnight, 0.25=sunrise, 0.5=noon, 0.75=sunset)
    time_of_day: f32,
    _pad2: f32,
    _pad3: f32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var t_atlas: texture_2d<f32>;
@group(1) @binding(1) var s_atlas: sampler;

// ── Vertex ──────────────────────────────────────────────────────────────────

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    // Per-vertex lighting (computed here, interpolated to fragment)
    @location(1) light: f32,
    // Fog factor (0 = no fog, 1 = fully fogged)
    @location(2) fog_factor: f32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_pos = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.tex_coords = in.tex_coords;

    // ── Lighting (per-vertex, cheap) ────────────────────────────────────

    // Directional sun light: dot(normal, sun_dir)
    let sun_dot = max(dot(in.normal, camera.sun_dir), 0.0);

    // Warm sun (slightly yellow) vs cool shadow (slightly blue)
    // Encoded as single brightness value, tint applied in fragment
    let sun_strength = sun_dot * 0.55;

    // Ambient: base light so shadows aren't pure black
    // Slightly stronger on top faces (sky ambient)
    let sky_ambient = select(0.35, 0.45, in.normal.y > 0.5);

    // Simple AO approximation from normal direction:
    // Bottom faces slightly darker, side faces medium, top faces brightest
    let ao = select(
        select(0.9, 0.8, in.normal.y < -0.5),  // side or bottom
        1.0,                                       // top
        in.normal.y > 0.5
    );

    // Day/night factor: reduce light at night
    // time_of_day: 0.5 = noon (brightest), 0.0 = midnight (darkest)
    let noon_dist = abs(camera.time_of_day - 0.5) * 2.0; // 0 at noon, 1 at midnight
    let day_factor = 1.0 - noon_dist * 0.6; // 1.0 at noon, 0.4 at midnight

    out.light = (sky_ambient + sun_strength) * ao * day_factor;

    // Clamp to reasonable range, slight brightness boost
    out.light = clamp(out.light * 1.15, 0.08, 1.0);

    // ── Fog ─────────────────────────────────────────────────────────────

    let dist = distance(in.position, camera.camera_pos);
    out.fog_factor = clamp(
        (dist - camera.fog_start) / (camera.fog_end - camera.fog_start),
        0.0, 1.0
    );
    // Smooth fog curve (quadratic, looks more natural than linear)
    out.fog_factor = out.fog_factor * out.fog_factor;

    return out;
}

// ── Fragment ────────────────────────────────────────────────────────────────

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture atlas
    let tex_color = textureSample(t_atlas, s_atlas, in.tex_coords);

    // Discard fully transparent pixels (leaves, glass cutout)
    if tex_color.a < 0.1 {
        discard;
    }

    // Apply lighting
    var color = tex_color.rgb * in.light;

    // Slight warm tint in lit areas, cool in shadow
    let warmth = in.light * 0.04;
    color = color + vec3<f32>(warmth, warmth * 0.5, -warmth * 0.3);

    // Apply fog
    color = mix(color, camera.fog_color, in.fog_factor);

    // Gamma correction (linear -> sRGB, atlas is already sRGB so light only)
    return vec4<f32>(color, tex_color.a);
}
