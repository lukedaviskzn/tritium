// Inputs

struct CameraUniform {
    view_pos: vec3<f32>,
    view_proj: mat4x4<f32>,
};

//!include("includes/colour_encodings.wgsl")
//!include("includes/tonemapping.wgsl")

//!binding()
var skybox_texture: texture_cube<f32>;
//!binding()
var skybox_sampler: sampler;
//!binding()
var<uniform> camera: CameraUniform;

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
    
    out.clip_position = camera.view_proj * (vec4(model.position + camera.view_pos, 1.0));
    // out.clip_position = camera.view_proj * vec4(model.position, 1.0);
    
    return out;
}

// Fragment Shader

struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
    @location(0) colour: vec4<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    
    var colour = textureSample(skybox_texture, skybox_sampler, in.position).rgb;

    // colour = tonemap(srgb_to_linear(colour));
    colour = tonemap(colour).rgb;
    // colour = linear_to_srgb(colour);

    var out: FragmentOutput;
    
    out.colour = vec4(colour, 1.0);
    out.depth = 1.0;

    return out;
}
