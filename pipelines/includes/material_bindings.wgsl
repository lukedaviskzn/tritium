struct AlphaMode {
    blended: u32,
    cutoff: f32,
}

struct Material {
    albedo: vec4<f32>,
    metallic: f32,
    roughness: f32,
    occlusion: f32,
    emissive: vec4<f32>,
}

//!binding()
var<uniform> alpha_mode: AlphaMode;

//!binding()
var albedo_texture: texture_2d<f32>;
//!binding()
var albedo_sampler: sampler;
//!binding()
var<uniform> albedo: vec4<f32>;

//!binding()
var metallic_texture: texture_2d<f32>;
//!binding()
var metallic_sampler: sampler;
//!binding()
var<uniform> metallic_factor: f32;
//!binding()
var roughness_texture: texture_2d<f32>;
//!binding()
var roughness_sampler: sampler;
//!binding()
var<uniform> roughness_factor: f32;

//!binding()
var normal_texture: texture_2d<f32>;
//!binding()
var normal_sampler: sampler;
//!binding()
var<uniform> normal_scale: f32;

//!binding()
var occlusion_texture: texture_2d<f32>;
//!binding()
var occlusion_sampler: sampler;
//!binding()
var<uniform> occlusion_strength: f32;

//!binding()
var emissive_texture: texture_2d<f32>;
//!binding()
var emissive_sampler: sampler;
//!binding()
var<uniform> emissive_factor: vec4<f32>;
