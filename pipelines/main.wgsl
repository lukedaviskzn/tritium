// Uniforms

@group(1) @binding(0)
var<uniform> model_matrix: mat4x4<f32>;

struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(2) @binding(0)
var<uniform> camera: CameraUniform;

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
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.position = normalize(model_matrix * vec4(model.position, 1.0)).xyz;
    out.normal = normalize(model_matrix * vec4(model.normal, 1.0)).xyz;
    out.tangent = normalize(model_matrix * vec4(model.tangent, 1.0)).xyz;
    out.bitangent = normalize(model_matrix * vec4(model.bitangent, 1.0)).xyz;
    
    out.clip_position = camera.view_proj * model_matrix * vec4(model.position, 1.0);
    
    return out;
}

// Fragment Shader

@group(0) @binding(0)
var diffuse_texture: texture_2d<f32>;
@group(0) @binding(1)
var diffuse_sampler: sampler;
@group(0) @binding(2)
var<uniform> diffuse_colour: vec4<f32>;
@group(0) @binding(3)
var normal_texture: texture_2d<f32>;
@group(0) @binding(4)
var normal_sampler: sampler;
@group(0) @binding(5)
var<uniform> normal_factor: f32;

struct Light {
    position: vec3<f32>,
    colour: vec4<f32>,
}
@group(3) @binding(0)
var<uniform> light: Light;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let colour: vec4<f32> = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords) * diffuse_colour;
    let map_normal: vec3<f32> = (textureSample(normal_texture, normal_sampler, in.tex_coords) * 2.0 - 1.0).xyz * normal_factor;

    let light_colour = light.colour.xyz * light.colour.w;
    
    // We don't need (or want) much ambient light, so 0.1 is fine
    let ambient_strength = 0.1;
    let ambient_colour = light_colour * ambient_strength;

    let tbn = mat3x3(in.tangent, in.bitangent, in.normal);
    let normal = normalize(tbn * map_normal);

    // Create the lighting vectors
    let light_dir = normalize(light.position - in.position);
    let view_dir = normalize(camera.view_pos.xyz - in.position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(normal, light_dir), 0.0);
    let diffuse_colour = light_colour * diffuse_strength;

    let specular_strength = pow(max(dot(normal, half_dir), 0.0), 32.0);
    let specular_colour = specular_strength * light_colour;

    let result = (ambient_colour + diffuse_colour + specular_colour) * colour.xyz;

    return vec4<f32>(result, colour.a);
}
