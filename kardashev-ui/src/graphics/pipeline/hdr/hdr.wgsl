
struct Config {
    tone_map: u32,
    exposure: f32,
    gamma: f32,
    padding: u32,
}

@group(0) @binding(0)
var<uniform> config: Config;


@group(1)
@binding(0)
var hdr_image: texture_2d<f32>;

@group(1)
@binding(1)
var hdr_sampler: sampler;

@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4f {
    let hdr = textureSample(hdr_image, hdr_sampler, vs.uv);

    var ldr: vec3f;
    switch config.tone_map {
        case 0u: {
            ldr = aces_tone_map(hdr.xyz);
        }
        case 1u: {
            ldr = reinard_tone_map(hdr.xyz);
        }
        case 2u: {
            ldr = exposure_tone_map(hdr.xyz, config.exposure);
        }
        default: {
            ldr = clamp(hdr.xyz, vec3f(0.0), vec3f(1.0));
        }
    }

    return vec4(gamma_correct(ldr, config.gamma), hdr.a);
}

// Maps HDR values to linear values
// Based on http://www.oscars.org/science-technology/sci-tech-projects/aces
fn aces_tone_map(hdr: vec3f) -> vec3f {
    let m1 = mat3x3(
        0.59719, 0.07600, 0.02840,
        0.35458, 0.90834, 0.13383,
        0.04823, 0.01566, 0.83777,
    );
    let m2 = mat3x3(
        1.60475, -0.10208, -0.00327,
        -0.53108,  1.10813, -0.07276,
        -0.07367, -0.00605,  1.07602,
    );
    let v = m1 * hdr;
    let a = v * (v + 0.0245786) - 0.000090537;
    let b = v * (0.983729 * v + 0.4329510) + 0.238081;
    return clamp(m2 * (a / b), vec3(0.0), vec3(1.0));
}

// https://learnopengl.com/Advanced-Lighting/HDR
fn reinard_tone_map(hdr: vec3f) -> vec3f {
    return hdr / (hdr + vec3f(1.0));
}

// https://learnopengl.com/Advanced-Lighting/HDR
fn exposure_tone_map(hdr: vec3f, exposure: f32) -> vec3f {
    return vec3(1.0) - exp(-hdr * exposure);
}

fn gamma_correct(sdr: vec3f, gamma: f32) -> vec3f {
    return pow(sdr, vec3(1.0 / gamma));
}

struct VertexOutput {
    @location(0) uv: vec2f,
    @builtin(position) clip_position: vec4f,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
) -> VertexOutput {
    var out: VertexOutput;
    // Generate a triangle that covers the whole screen
    out.uv = vec2f(
        f32((vi << 1u) & 2u),
        f32(vi & 2u),
    );
    out.clip_position = vec4f(out.uv * 2.0 - 1.0, 0.0, 1.0);
    // We need to invert the y coordinate so the image
    // is not upside down
    out.uv.y = 1.0 - out.uv.y;
    return out;
}
