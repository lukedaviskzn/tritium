
fn tonemap(colour: vec3<f32>) -> vec3<f32> {
    // let exposure = 0.5;
    // let constrast = 0.75;
    // return pow(exposure * colour, vec3(constrast));
    return colour / (colour + 1.0);
}

fn tonemap_rgba(colour: vec4<f32>) -> vec4<f32> {
    return vec4(tonemap(colour.rgb), colour.a);
}
