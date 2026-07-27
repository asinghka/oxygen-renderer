struct Camera {
    eye: vec3<f32>,
    view_projection: mat4x4<f32>,
}

struct Material {
    color: vec4<f32>,
    bump: f32,
}

struct Transform {
    model: mat4x4<f32>,
    normal_model: mat4x4<f32>,
    scale: f32,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> material: Material;

@group(2) @binding(0)
var<uniform> transform: Transform;

@vertex
fn vertex_shader(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    let custom_scale = scale_matrix(transform.scale);

    return camera.view_projection * custom_scale * transform.model * vec4<f32>(position, 1.0);;
}

fn scale_matrix(scale: f32) -> mat4x4<f32> {
    return mat4x4<f32>(vec4(scale, 0.0, 0.0, 0.0), vec4(0.0, scale, 0.0, 0.0), vec4(0.0, 0.0, scale, 0.0), vec4(0.0, 0.0, 0.0, 1.0));
}

@fragment
fn fragment_shader() -> @location(0) vec4<f32> {
    return material.color;
}