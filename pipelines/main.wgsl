// Inputs

let PI: f32 = 3.14159265358979323846264338327950288;

struct Transform {
    model_matrix: mat4x4<f32>,
    inv_model_matrix: mat4x4<f32>,
}

struct Camera {
    position: vec4<f32>,
    view_proj: mat4x4<f32>,
};

// struct PointLight {
//     position: vec3<f32>,
//     colour: vec4<f32>,
// }

// struct DirectionalLight {
//     direction: vec3<f32>,
//     colour: vec4<f32>,
// }

struct Light {
    pos_dir: vec3<f32>,
    colour: vec4<f32>,
}

struct AmbientLight {
    colour: vec4<f32>,
}

//!include("includes/colour_encodings.wgsl")
//!include("includes/tonemapping.wgsl")
//!include("includes/material_bindings.wgsl")

//!binding()
var<uniform> transform: Transform;
//!binding()
var<uniform> camera: Camera;
//!binding()
var<storage> point_lights: array<Light>;
//!binding()
var<uniform> num_point_lights: u32;
//!binding()
var<storage> directional_lights: array<Light>;
//!binding()
var<uniform> num_directional_lights: u32;
//!binding()
var<storage> ambient_lights: array<AmbientLight>;
//!binding()
var<uniform> num_ambient_lights: u32;
//!binding()
var irradiance_map: texture_cube<f32>;
//!binding()
var irradiance_sampler: sampler;
//!binding()
var reflections_map: texture_cube<f32>;
//!binding()
var reflections_sampler: sampler;

// Vertex Shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) position: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let pos = transform.model_matrix * vec4(model.position, 1.0);
    
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.position = pos.xyz;
    out.normal = normalize(transpose(transform.inv_model_matrix) * vec4(model.normal, 0.0)).xyz;
    out.tangent = normalize(transform.model_matrix * vec4(model.tangent, 0.0)).xyz;
    out.bitangent = normalize(transform.model_matrix * vec4(model.bitangent, 0.0)).xyz;

    out.clip_position = camera.view_proj * pos;
    
    return out;
}

// Fragment Shader

fn distribution_ggx(normal: vec3<f32>, half: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let ndoth = max(dot(normal, half), 0.0);
    let ndoth2 = ndoth * ndoth;
	
    let num = a2;
    let denom = (ndoth2 * (a2 - 1.0) + 1.0);
    let denom = PI * denom * denom;
	
    return num / denom;
}

fn geometry_schlick_ggx(ndotv: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r*r) / 8.0;

    let num = ndotv;
    let denom = ndotv * (1.0 - k) + k;
	
    return num / denom;
}

fn geometry_smith(normal: vec3<f32>, view: vec3<f32>, light_dir: vec3<f32>, roughness: f32) -> f32{
    let ndotv = max(dot(normal, view), 0.0);
    let ndotl = max(dot(normal, light_dir), 0.0);
    let ggx_v  = geometry_schlick_ggx(ndotv, roughness);
    let ggx_l  = geometry_schlick_ggx(ndotl, roughness);
	
    return ggx_v * ggx_l;
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn fresnel_schlick_roughness(cos_theta: f32, f0: vec3<f32>, roughness: f32) -> vec3<f32> {
    return f0 + (max(vec3(1.0 - roughness), f0) - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn calc_light(light: Light, directional: bool, material: Material, position: vec3<f32>, normal: vec3<f32>, view_dir: vec3<f32>) -> vec3<f32> {
    let light_colour = light.colour.rgb * light.colour.a;

    let light_vec = light.pos_dir - position;
    
    var light_dir = vec3(0.0);
    if (directional) {
        light_dir = light.pos_dir;
    } else {
        light_dir = normalize(light_vec);
    };
    
    let half_dir = normalize(view_dir + light_dir);
    
    var attenuation = 1.0;
    if (!directional) {
        let distance = length(light_vec);
        attenuation /= distance * distance + 1.0;
    };
    
    let radiance = light_colour * attenuation;    
    
    let f0 = vec3(0.04);
    let f0 = mix(f0, material.albedo.rgb, material.metallic);
    let f = fresnel_schlick(max(dot(half_dir, view_dir), 0.0), f0);
    let ndf = distribution_ggx(normal, half_dir, material.roughness);
    let g = geometry_smith(normal, view_dir, light_dir, material.roughness);

    let ks = f;
    let kd = (vec3(1.0) - ks) * (1.0 - material.metallic);
    
    let numerator = ndf * g * f;
    let denominator = 4.0 * max(dot(normal, view_dir), 0.0) * max(dot(normal, light_dir), 0.0)  + 0.0001;
    let specular = numerator / denominator;
    
    let ndotl = max(dot(normal, light_dir), 0.0);        
    return (kd * material.albedo.rgb / PI + specular) * radiance * ndotl;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let map_normal = (textureSample(normal_texture, normal_sampler, in.tex_coords) * 2.0 - 1.0).rgb;
    let map_normal = mix(vec3(0.0, 0.0, 1.0), map_normal, normal_scale);
    // let map_normal = vec3(0.0, 0.0, 1.0);

    var material: Material;
    material.albedo = textureSample(albedo_texture, albedo_sampler, in.tex_coords) * albedo;
    material.albedo = srgb_to_linear(material.albedo) * srgb_to_linear(albedo);
    
    material.metallic = textureSample(metallic_texture, metallic_sampler, in.tex_coords).b * metallic_factor;
    material.roughness = textureSample(roughness_texture, roughness_sampler, in.tex_coords).g * roughness_factor;
    material.occlusion = mix(1.0, textureSample(occlusion_texture, occlusion_sampler, in.tex_coords).r, occlusion_strength);
    material.emissive = textureSample(emissive_texture, emissive_sampler, in.tex_coords) * emissive_factor;

    let tbn = mat3x3(in.tangent, in.bitangent, in.normal);
    let normal = normalize(tbn * map_normal);
    let view_dir = normalize(camera.position.xyz - in.position);

    let f0 = vec3(0.04);
    let f0 = mix(f0, material.albedo.rgb, material.metallic);
    let ks = fresnel_schlick_roughness(max(dot(normal, view_dir), 0.0), f0, material.roughness); 
    let kd = 1.0 - ks;
    let irradiance = textureSample(irradiance_map, irradiance_sampler, normal).rgb;
    let diffuse = irradiance * material.albedo.rgb;
    let ambient = (kd * diffuse) * material.occlusion; 
    // todo: fix this, this is not the proper way of doing reflections
    let r = 2.0 * dot(view_dir, normal) * normal - view_dir;
    let reflections = textureSample(reflections_map, reflections_sampler, r).rgb;
    let reflections = reflections * material.albedo.rgb;
    let ambient = ambient + reflections;

    if (material.albedo.a <= alpha_mode.cutoff) {
        discard;
    }

    if (alpha_mode.blended == 0u) {
        material.albedo.a = 1.0;
    }

    var reflectance = vec3(0.0);
    for (var i = 0u; i < num_point_lights; i++) {
        reflectance += calc_light(point_lights[i], false, material, in.position, normal, view_dir);
    }
    
    for (var i = 0u; i < num_directional_lights; i++) {
        reflectance += calc_light(directional_lights[i], true, material, in.position, normal, view_dir);
    }
    
    var ambient_light = vec3(0.0) + ambient;
    for (var i = 0u; i < num_ambient_lights; i++) {
        ambient_light += ambient_lights[i].colour.rgb * ambient_lights[i].colour.a * material.albedo.rgb;
    }
    ambient_light *= material.occlusion;

    var final_colour = reflectance + ambient_light + material.emissive.rgb;
    
    // final_colour /= (final_colour + vec3(1.0));

    let final_colour = tonemap(final_colour);
    
    return vec4(final_colour, material.albedo.a);
    // return vec4(vec3(material.metallic), 1.0);
    // return vec4(pow((in.normal + 1.0) / 2.0, vec3(2.2)), 1.0);
    // return vec4(vec3(normal_scale / 2.0), 1.0);
}
