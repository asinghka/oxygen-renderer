struct RenderSettings {
    ambient: f32,
    diffuse: u32,
    specular: u32,
    specular_strength: f32,
    specular_exponent: f32,
    bump: f32,
    shadow: u32,
}

struct Camera {
    eye: vec3<f32>,
    view_projection: mat4x4<f32>,
}

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
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> settings: RenderSettings;

@group(2) @binding(0)
var<uniform> light: Light;

@group(2) @binding(1)
var shadow_map_sampler: sampler_comparison;

@group(2) @binding(2)
var shadow_map_texel: texture_depth_2d;

@group(3) @binding(0)
var<uniform> primitive: Primitive;

@group(3) @binding(1)
var tex_sampler: sampler;

@group(3) @binding(2)
var albedo_texel: texture_2d<f32>;

@group(3) @binding(3)
var normal_texel: texture_2d<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) light_pos: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) tangent: vec4<f32>,
}

@vertex
fn vertex_shader(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let model = primitive.model;
    let normal_model = primitive.normal_model;

    let world_pos = model * vec4<f32>(in.position, 1.0);
    out.world_pos = world_pos.xyz;

    out.light_pos = light.view_ortho * world_pos;

    out.normal = (normal_model * vec4<f32>(in.normal, 0.0)).xyz;
    out.tangent = vec4<f32>((model * vec4<f32>(in.tangent.xyz, 0.0)).xyz, in.tangent.w);
    out.clip_pos = camera.view_projection * world_pos;
    out.uv = in.uv;

    return out;
}

@fragment
fn fragment_shader(in: VertexOutput) -> @location(0) vec4<f32> {
    var n = normalize(in.normal);

    let t = normalize(in.tangent.xyz);
    let b = cross(n, t) * in.tangent.w;
    let tbn = mat3x3<f32>(t, b, n);

    var bump = primitive.bump * settings.bump;

    n = textureSample(normal_texel, tex_sampler, in.uv).xyz;
    n = 2.0 * n - 1.0;
    n = vec3<f32>(bump, bump, 1.0) * n;
    n = normalize(tbn * n);

    let light_dir = normalize(light.direction);

    let ndc = in.light_pos.xyz / in.light_pos.w;
    let uv  = ndc.xy * vec2(0.5, -0.5) + vec2(0.5, 0.5);

    var shadow = 1.0;
    if (all(uv >= vec2(0.0)) && all(uv <= vec2(1.0)) && ndc.z <= 1.0 && settings.shadow == 1u) {
        shadow = textureSampleCompareLevel(shadow_map_texel, shadow_map_sampler, uv, ndc.z);
    }

    let ambient = settings.ambient;

    let albedo = textureSample(albedo_texel, tex_sampler, in.uv).rgb * primitive.color;

    let n_dot_l = dot(n, light_dir);

    var diffuse = 0.0;
    if settings.diffuse != 0u {
        diffuse = max(n_dot_l, 0.0);
    }
    diffuse = diffuse * shadow;

    let view_dir = normalize(camera.eye - in.world_pos);
    let half_dir = normalize(light_dir + view_dir);

    var specular = 0.0;
    if settings.specular != 0u && n_dot_l > 0.0 { // Make sure specular glint is only visible when facing the light
        specular = settings.specular_strength * pow(max(dot(n, half_dir), 0.0), settings.specular_exponent);
    }
    specular = specular * shadow;

    let color = albedo * (ambient + diffuse) + specular;

    return vec4<f32>(color, 1.0);
}
