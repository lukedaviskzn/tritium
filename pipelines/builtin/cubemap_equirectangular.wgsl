// Inputs

//!include("../includes/colour_encodings.wgsl")

//!binding()
var in_texture: texture_2d<f32>;
//!binding()
var in_sampler: sampler;
//!binding()
var<uniform> view_proj: mat4x4<f32>;

// Vertex Shader

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = model.position;
    out.clip_position = view_proj * vec4(model.position, 1.0);
    
    return out;
}

// Fragment Shader

fn spherical_to_equirectangular(v: vec3<f32>) -> vec2<f32> {
    var uv = vec2(atan2(v.z, v.x), asin(v.y));
    let inv_atan = vec2(0.1591, 0.3183);
    uv *= inv_atan;
    uv += 0.5;
    return uv;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var colour = textureSample(in_texture, in_sampler, spherical_to_equirectangular(normalize(in.position)));

    // colour = colour / (colour + vec4(1.0));
    // colour = tonemap_rgba(srgba_to_linear(colour));
    colour = srgba_to_linear(colour); 

    return colour;
}
