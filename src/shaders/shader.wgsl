struct CameraUniform {
    view_projection: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
}

@vertex
fn vertex_shader(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.normal = in.normal;
    out.clip_position = camera.view_projection * vec4<f32>(in.position, 1.0);

    return out;
}

@fragment
fn fragment_shader(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);

    let light_dir = normalize(vec3<f32>(0.4, 0.8, 0.6));

    let diffuse = max(dot(n, light_dir), 0.0);
    let ambient = 0.1;
    let albedo = vec3<f32>(1.0, 0.5, 0.2);

    let color = albedo * (ambient + diffuse);

    return vec4<f32>(color, 1.0);
}
