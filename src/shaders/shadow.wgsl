struct Light {
    direction: vec3<f32>,
    view_ortho: mat4x4<f32>,
}

struct Primitive {
    model: mat4x4<f32>,
    normal_model: mat4x4<f32>,
    color: vec3<f32>,
    bump: f32,
}

@group(0) @binding(0)
var<uniform> light: Light;

@group(1) @binding(0)
var<uniform> primitive: Primitive;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

@vertex
fn vertex_shader(in: VertexInput) -> @builtin(position) vec4<f32> {
    return light.view_ortho * primitive.model * vec4<f32>(in.position, 1.0);
}