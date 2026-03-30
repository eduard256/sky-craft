// Sky Craft block shader -- vertex + fragment.
// Renders colored voxel faces with simple directional lighting.

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    out.normal = in.normal;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple directional light from above-right
    let light_dir = normalize(vec3<f32>(0.3, 1.0, 0.5));
    let ambient = 0.4;
    let diffuse = max(dot(in.normal, light_dir), 0.0) * 0.6;
    let brightness = ambient + diffuse;

    let lit_color = in.color * brightness;
    return vec4<f32>(lit_color, 1.0);
}
