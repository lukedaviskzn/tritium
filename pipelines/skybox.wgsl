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
    
    out.clip_position = camera.view_proj * model_matrix * vec4(model.position, 1.0);
    
    return out;
}

// Fragment Shader

@group(0) @binding(0)
var skybox_texture: texture_cube<f32>;
@group(0) @binding(1)
var skybox_sampler: sampler;

struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
    @location(0) colour: vec4<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    
    out.colour = textureSample(skybox_texture, skybox_sampler, in.position);
    out.depth = 1.0;

    return out;
}
