let PI: f32 = 3.14159265358979323846264338327950288;

//!binding()
var irradiance_map: texture_cube<f32>;
//!binding()
var irradiance_sampler: sampler;
//!binding()
var<uniform> roughness: f32;

fn radical_inverse_vdc(bits: u32) -> f32 {
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return f32(bits) * 2.3283064365386963e-10; // / 0x100000000
}

fn hammersley(i: u32, n: u32) -> vec2<f32> {
    return vec2(f32(i)/f32(n), radical_inverse_vdc(i));
}  

fn importance_sample_ggx(xi: vec2<f32>, n: vec3<f32>, roughness: f32) -> vec3<f32> {
    let a = roughness * roughness;
	
    let phi = 2.0 * PI * xi.x;
    let cos_theta = sqrt((1.0 - xi.y) / (1.0 + (a*a - 1.0) * xi.y));
    let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
	
    // from spherical coordinates to cartesian coordinates
    let h = vec3(cos(phi) * sin_theta, sin(phi) * sin_theta, cos_theta);
	
    // from tangent-space vector to world-space sample vector
    let up = abs(n.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
    let tangent = normalize(cross(up, n));
    let bitangent = cross(n, tangent);
	
    let sample_vec = tangent * h.x + bitangent * h.y + n * h.z;
    return normalize(sample_vec);
}  

@vertex
fn vs_main(
    @location(0) position: vec3<f32>
) -> @builtin(position) vec4<f32> {
    return view_proj * vec4(position, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let n = normalize(position);    
    let r = n;
    let v = n;

    let sample_count = 1024u;
    let total_weight = 0.0;   
    let prefiltered_colour = vec3(0.0);     
    for(let i = 0u; i < sample_count; i++) {
        vec2 xi = hammersley(i, sample_count);
        vec3 h  = importance_sample_ggx(xi, n, roughness);
        vec3 l  = normalize(2.0 * dot(v, h) * h - v);

        let ndotl = max(dot(n, l), 0.0);
        if (ndotl > 0.0) {
            prefiltered_colour += textureSample(irradiance_map, irradiance_sampler, l).rgb * ndotl;
            total_weight += ndotl;
        }
    }
    prefiltered_colour /= total_weight;

    return vec4(prefiltered_colour, 1.0);
}  