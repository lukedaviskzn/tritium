
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    // var out: VertexOutput;
    // let x = f32(1 - i32(in_vertex_index)) * 0.5;
    // let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    // out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    // return out;

    var out: VertexOutput;
    
    out.tex_coords = vec2(f32((in_vertex_index << 1u) & 2u), f32(in_vertex_index & 2u));
    out.clip_position = vec4(vec2(out.tex_coords.x, out.tex_coords.y) * 2.0 - 1.0, 0.0, 1.0);
    out.tex_coords.y = 1.0 - out.tex_coords.y;

    return out;
}

//!binding()
var texture_in: texture_2d<f32>;
//!binding()
var sampler_in: sampler;

@fragment
fn fs_main(vout: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture_in, sampler_in, vout.tex_coords);
}
