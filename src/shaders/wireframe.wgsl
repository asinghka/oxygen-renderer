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
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> material: Material;

@group(2) @binding(0)
var<uniform> transform: Transform;

@vertex
fn vertex_shader(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return camera.view_projection * transform.model * vec4<f32>(position, 1.0);;
}

@fragment
fn fragment_shader() -> @location(0) vec4<f32> {
    return material.color;
}