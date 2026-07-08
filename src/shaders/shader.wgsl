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

struct RenderSettings {
    ambient: f32,
    diffuse: u32,
    color: vec3<f32>,
}

@group(1) @binding(0)
var<uniform> settings: RenderSettings;

@fragment
fn fragment_shader(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);

    let light_dir = normalize(vec3<f32>(0.4, 0.8, 0.6));

    var diffuse = 0.0;
    if settings.diffuse != 0u {
        diffuse = max(dot(n, light_dir), 0.0);
    }

    let ambient = settings.ambient;
    let albedo = settings.color;

    let color = albedo * (ambient + diffuse);

    return vec4<f32>(color, 1.0);
}
