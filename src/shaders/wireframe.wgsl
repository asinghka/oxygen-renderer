struct Camera {
    eye: vec3<f32>,
    view_projection: mat4x4<f32>,
}

struct Primitive {
    model: mat4x4<f32>,
    normal_model: mat4x4<f32>,
    color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> primitive: Primitive;

@vertex
fn vertex_shader(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return camera.view_projection * primitive.model * vec4<f32>(position, 1.0);;
}

@fragment
fn fragment_shader() -> @location(0) vec4<f32> {
    return primitive.color;
}