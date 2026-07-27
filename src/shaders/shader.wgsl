struct Camera {
    eye: vec3<f32>,
    view_projection: mat4x4<f32>,
}

struct Transform {
    model: mat4x4<f32>,
    normal_model: mat4x4<f32>,
    scale: f32,
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
    pcf: f32,
    pbr: u32,
    depth: u32,
    normal: u32,
}

struct Light {
    direction: vec3<f32>,
    view_ortho: mat4x4<f32>,
}

struct Material {
    color: vec4<f32>,
    bump: f32,
    metallic: f32,
    roughness: f32,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(0) @binding(1)
var<uniform> settings: RenderSettings;

@group(0) @binding(2)
var<uniform> light: Light;

@group(0) @binding(3)
var shadow_map_sampler: sampler_comparison;

@group(0) @binding(4)
var shadow_map_texel: texture_depth_2d;

@group(1) @binding(0)
var<uniform> material: Material;

@group(1) @binding(1)
var albedo_texel: texture_2d<f32>;

@group(1) @binding(2)
var normal_texel: texture_2d<f32>;

@group(1) @binding(3)
var tex_sampler: sampler;

@group(2) @binding(0)
var<uniform> transform: Transform;

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

    let custom_scale = scale_matrix(transform.scale);

    let model = custom_scale * transform.model;
    let normal_model = transform.normal_model;

    let world_pos = model * vec4<f32>(in.position, 1.0);
    out.world_pos = world_pos.xyz;

    out.light_pos = light.view_ortho * world_pos;
    out.normal = (normal_model * vec4<f32>(in.normal, 0.0)).xyz;
    out.tangent = vec4<f32>((model * vec4<f32>(in.tangent.xyz, 0.0)).xyz, in.tangent.w);
    out.clip_pos = camera.view_projection * world_pos;
    out.uv = in.uv;

    return out;
}

fn scale_matrix(scale: f32) -> mat4x4<f32> {
    return mat4x4<f32>(vec4(scale, 0.0, 0.0, 0.0), vec4(0.0, scale, 0.0, 0.0), vec4(0.0, 0.0, scale, 0.0), vec4(0.0, 0.0, 0.0, 1.0));
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

    let albedo = textureSample(albedo_texel, tex_sampler, uv).rgb * material.color.rgb;
    let shadow = sample_shadow(in.light_pos);

    var color = vec3<f32>(0.0, 0.0, 0.0);
    if settings.pbr == 1u {
        color = physically_based_lighting(normal, light.direction, camera.eye, in.world_pos, albedo, shadow);
    } else {
        color = blinn_phong_lighting(normal, light.direction, camera.eye, in.world_pos, albedo, shadow);
    }

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

    let extent = settings.pcf * 0.5;
    let samples = pow(settings.pcf + 1.0, 2.0);
    let resolution = settings.shadow_map_resolution;

    for (var y = -extent; y <= extent; y += 1.0) {
        for (var x = -extent; x <= extent; x += 1.0) {
            let uv_offset = uv + vec2(x / resolution, y / resolution);
            if (all(uv_offset >= vec2(0.0)) && all(uv_offset <= vec2(1.0)) && ndc.z <= 1.0) {
                shadow += textureSampleCompareLevel(shadow_map_texel, shadow_map_sampler, uv_offset, ndc.z);
            }
        }
    }

    return shadow / samples;
}

fn apply_normal_map(normal: vec3<f32>, tangent: vec4<f32>, uv: vec2<f32>) -> vec3<f32> {
    let normal_strength = material.bump * settings.bump;
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

const PI: f32 = 3.1415926;

fn physically_based_lighting(normal: vec3<f32>, light_dir: vec3<f32>, camera_eye: vec3<f32>, world_space_pos: vec3<f32>, albedo: vec3<f32>, shadow: f32) -> vec3<f32> {
    let n = normalize(normal);
    let l = normalize(light_dir);
    let v = normalize(camera_eye - world_space_pos);
    let h = normalize(l + v);

    let n_dot_l = max(dot(n, l), 0.0);
    let n_dot_v = max(dot(n, v), 0.0);
    let n_dot_h = max(dot(n, h), 0.0);
    let h_dot_v = max(dot(h, v), 0.0);

    let roughness = clamp(material.roughness, 0.045, 1.0);
    let k = pow(roughness + 1.0, 2.0) / 8.0;
    let f0 = mix(vec3<f32>(0.04), albedo, material.metallic);

    let d = distribution_ggx(n_dot_h, roughness);
    let g = geometry_smith(n_dot_v, n_dot_l, k);
    let f = fresnel_schlick(f0, h_dot_v);

    let specular = (d * g * f) / (4.0 * n_dot_v * n_dot_l + 0.0001);
    let kd = (vec3<f32>(1.0) - f) * (1.0 - material.metallic);
    let diffuse = kd * albedo / PI;

    let radiance_out = (diffuse + specular) * n_dot_l * shadow;
    let ambient = vec3<f32>(settings.ambient) * albedo;
    let color = ambient + radiance_out;

    return color;
}

fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;
    let n_dot_h2 = n_dot_h * n_dot_h;

    let denominator = n_dot_h2 * (alpha2 - 1.0) + 1.0;

    return alpha2 / (PI * denominator * denominator);
}

fn geometry_schlick_ggx(n_dot_x: f32, k: f32) -> f32 {
    return n_dot_x / (n_dot_x * (1.0 - k) + k);
}

fn geometry_smith(n_dot_v: f32, n_dot_l: f32, k: f32) -> f32 {
    return geometry_schlick_ggx(n_dot_v, k) * geometry_schlick_ggx(n_dot_l, k);
}

fn fresnel_schlick(f0: vec3<f32>, h_dot_v: f32) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - h_dot_v, 5.0);
}