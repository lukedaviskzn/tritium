
fn srgba_to_linear(colour: vec4<f32>) -> vec4<f32> {
    return vec4(pow(colour.rgb, vec3(2.2)), colour.a);
}

fn linear_to_srgba(colour: vec4<f32>) -> vec4<f32> {
    return vec4(pow(colour.rgb, vec3(1.0/2.2)), colour.a);
}

fn srgb_to_linear(colour: vec3<f32>) -> vec3<f32> {
    return pow(colour.rgb, vec3(2.2));
}

fn linear_to_srgb(colour: vec3<f32>) -> vec3<f32> {
    return pow(colour.rgb, vec3(1.0/2.2));
}
