// Vertex Shader

struct Transform {
    model_matrix: mat4x4<f32>,
    inv_model_matrix: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> transform: Transform;

struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct Light {
    position: vec3<f32>,
    colour: vec4<f32>,
}
@group(2) @binding(0)
var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) colour: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * transform.model_matrix * vec4(model.position, 1.0);
    out.colour = light.colour.rgb * light.colour.a;
    return out;
}

// Fragment Shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.colour, 1.0);
}
