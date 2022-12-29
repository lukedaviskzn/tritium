// Inputs

struct Transform {
    model_matrix: mat4x4<f32>,
    inv_model_matrix: mat4x4<f32>,
}

struct CameraUniform {
    position: vec4<f32>,
    view_proj: mat4x4<f32>,
};

struct PointLight {
    position: vec3<f32>,
    colour: vec4<f32>,
}

struct DirectionalLight {
    direction: vec3<f32>,
    colour: vec4<f32>,
}

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
@group(0) @binding(6)
var<uniform> transform: Transform;
@group(0) @binding(7)
var<uniform> camera: CameraUniform;
@group(0) @binding(8)
var<storage> point_lights: array<PointLight>;
@group(0) @binding(9)
var<uniform> num_point_lights: u32;
@group(0) @binding(10)
var<storage> directional_lights: array<DirectionalLight>;
@group(0) @binding(11)
var<uniform> num_directional_lights: u32;

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

fn calc_point_light(light: PointLight, position: vec3<f32>, normal: vec3<f32>, view_dir: vec3<f32>, colour: vec4<f32>) -> vec3<f32> {
    let light_colour = light.colour.xyz * light.colour.w;
    
    // We don't need (or want) much ambient light, so 0.1 is fine
    let ambient_strength = 0.1;
    let ambient_colour = light_colour * ambient_strength;

    // Create the lighting vectors
    let light_vec = light.position - position;
    let light_dir = normalize(light_vec);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(normal, light_dir), 0.0);
    let diffuse_colour = light_colour * diffuse_strength;

    let specular_strength = pow(max(dot(normal, half_dir), 0.0), 32.0);
    let specular_colour = specular_strength * light_colour;

    let attenuation = length(light_vec) * length(light_vec);

    let result = (ambient_colour + diffuse_colour + specular_colour) * colour.xyz / attenuation;

    return result;
}

fn calc_directional_light(light: DirectionalLight, position: vec3<f32>, normal: vec3<f32>, view_dir: vec3<f32>, colour: vec4<f32>) -> vec3<f32> {
    let light_colour = light.colour.xyz * light.colour.w;
    
    // We don't need (or want) much ambient light, so 0.1 is fine
    let ambient_strength = 0.1;
    let ambient_colour = light_colour * ambient_strength;

    // Create the lighting vectors
    let light_dir = normalize(light.direction);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(normal, light_dir), 0.0);
    let diffuse_colour = light_colour * diffuse_strength;

    let specular_strength = pow(max(dot(normal, half_dir), 0.0), 32.0);
    let specular_colour = specular_strength * light_colour;

    let result = (ambient_colour + diffuse_colour + specular_colour) * colour.xyz;

    return result;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let colour: vec4<f32> = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords) * diffuse_colour;
    let map_normal: vec3<f32> = (textureSample(normal_texture, normal_sampler, in.tex_coords) * 2.0 - 1.0).xyz * normal_factor;

    let tbn = mat3x3(in.tangent, in.bitangent, in.normal);
    let normal = normalize(tbn * map_normal);
    let view_dir = normalize(camera.position.xyz - in.position);

    var final_colour = vec3(0.0);
    
    for (var i = 0u; i < num_point_lights; i++) {
        final_colour += calc_point_light(point_lights[i], in.position, normal, view_dir, colour);
    }
    
    for (var i = 0u; i < num_directional_lights; i++) {
        final_colour += calc_directional_light(directional_lights[i], in.position, normal, view_dir, colour);
    }
    
    return vec4(final_colour, colour.a);
    // return vec4((normal + 1.0) / 2.0, 1.0);
}
