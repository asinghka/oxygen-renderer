struct Camera {
    eye: vec3<f32>,
    view_projection: mat4x4<f32>,
}

struct RenderSettings {
    ambient: f32,
    diffuse: u32,
    specular: u32,
    specular_strength: f32,
    specular_exponent: f32,
    bump: f32,
    shadow: u32,
    shadow_map_resolution: f32,
    pcf: u32,
    depth: u32,
    normal: u32,
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
    if settings.depth == 1u {
        return depth_color(in.clip_pos);
    }

    let uv = in.uv;
    let normal = apply_normal_map(in.normal, in.tangent, uv);

    if settings.normal == 1u {
        return normal_color(normal);
    }

    let albedo = textureSample(albedo_texel, tex_sampler, uv).rgb * primitive.color;

    let shadow = sample_shadow(in.light_pos);
    let color = blinn_phong_lighting(normal, light.direction, camera.eye, in.world_pos, albedo, shadow);

    return vec4<f32>(color, 1.0);
}

fn blinn_phong_lighting(normal: vec3<f32>, light_dir: vec3<f32>, camera_eye: vec3<f32>, world_space_pos: vec3<f32>, albedo: vec3<f32>, shadow: f32) -> vec3<f32> {
    let n_dot_l = dot(normalize(normal), normalize(light_dir));

    let ambient = settings.ambient;
    let diffuse = diffuse(n_dot_l) * shadow;
    let specular = specular(camera_eye, light_dir, normal, n_dot_l, world_space_pos) * shadow;

    return albedo * (ambient + diffuse) + specular;
}

fn diffuse(n_dot_l: f32) -> f32 {
    var diffuse = 0.0;
    if settings.diffuse != 0u {
        diffuse = max(n_dot_l, 0.0);
    }

    return diffuse;
}

fn specular(camera_eye: vec3<f32>, light_dir: vec3<f32>, normal: vec3<f32>, n_dot_l: f32, world_space_pos: vec3<f32>) -> f32 {
    let view_dir = normalize(camera_eye - world_space_pos);
    let half_dir = normalize(light_dir + view_dir);

    var specular = 0.0;
    if settings.specular != 0u && n_dot_l > 0.0 { // Make sure specular glint is only visible when facing the light
        specular = settings.specular_strength * pow(max(dot(normal, half_dir), 0.0), settings.specular_exponent);
    }

    return specular;
}

fn sample_shadow(light_space_pos: vec4<f32>) -> f32 {
    if (settings.shadow == 0u) {
        return 1.0;
    }

    let ndc = light_space_pos.xyz / light_space_pos.w;
    let uv  = ndc.xy * vec2(0.5, -0.5) + vec2(0.5, 0.5);

    var shadow = 0.0;
    var count = 0.0;

    let samples = f32(settings.pcf);
    for (var y = -1.5 * samples; y <= 1.5 * samples; y += 1.0) {
        for (var x = -1.5 * samples; x <= 1.5 * samples; x += 1.0) {
            count += 1.0;
            let resolution = settings.shadow_map_resolution;
            let uv_offset = uv + vec2(x / resolution, y / resolution);
            if (all(uv_offset >= vec2(0.0)) && all(uv_offset <= vec2(1.0)) && ndc.z <= 1.0) {
                shadow += textureSampleCompareLevel(shadow_map_texel, shadow_map_sampler, uv_offset, ndc.z);
            }
        }
    }

    return shadow / count;
}

fn apply_normal_map(normal: vec3<f32>, tangent: vec4<f32>, uv: vec2<f32>) -> vec3<f32> {
    let normal_strength = primitive.bump * settings.bump;
    if normal_strength == 0.0 {
        return normalize(normal);
    }

    let world_normal = normalize(normal);
    let world_tangent = normalize(tangent.xyz - world_normal * dot(world_normal, tangent.xyz));
    let world_bitangent = normalize(cross(world_normal, world_tangent) * tangent.w);
    let tangent_to_world = mat3x3<f32>(world_tangent, world_bitangent, world_normal);

    let sampled_normal = textureSample(normal_texel, tex_sampler, uv).xyz;
    let tangent_space_normal = sampled_normal * 2.0 - vec3<f32>(1.0);
    let scaled_tangent_space_normal = normalize(vec3<f32>(
        tangent_space_normal.xy * normal_strength,
        tangent_space_normal.z,
    ));

    return normalize(tangent_to_world * scaled_tangent_space_normal);
}

// Inverse of the sRGB OETF the target applies on write.
// Debug modes output data, not light
fn linear_from_gamma(s: vec3<f32>) -> vec3<f32> {
    let lower = s / 12.92;
    let higher = pow((s + 0.055) / 1.055, vec3<f32>(2.4));

    return select(higher, lower, s <= vec3<f32>(0.04045));
}

fn depth_color(clip_pos: vec4<f32>) -> vec4<f32> {
    let near = 0.5;
    let far = 5.0;

    let z = clip_pos.z;
    let t = saturate((near * z) / (far - z * (far - near)));
    return vec4<f32>(linear_from_gamma(vec3<f32>(t)), 1.0);
}

fn normal_color(normal: vec3<f32>) -> vec4<f32> {
    return vec4<f32>(linear_from_gamma(0.5 * normal + 0.5), 1.0);
}