#import ../globals.wgsl::Globals;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
}

struct FragmentOutput {
    @location(0) color: vec4f,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

struct Debug {
    channel: u32,
    _padding1: u32,
    _padding2: u32,
    _padding3: u32,
}

@group(2) @binding(0)
var<uniform> debug: Debug;

@group(1) @binding(0)
var texture_sampler: sampler;
@group(1) @binding(1)
var position_texture: texture_2d<f32>;
@group(1) @binding(2)
var normal_texture: texture_2d<f32>;
@group(1) @binding(3)
var diffuse_occlusion_texture: texture_2d<f32>;
@group(1) @binding(4)
var specular_shininess_texture: texture_2d<f32>;
@group(1) @binding(5)
var emission_texture: texture_2d<f32>;

@vertex
fn vs_main(
    in: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = vec2<f32>(
        f32((in.vertex_index << 1u) & 2u),
        f32(in.vertex_index & 2u),
    );
    out.clip_position = vec4<f32>(out.tex_coords * 2.0 - 1.0, 0.0, 1.0);
    out.tex_coords.y = 1.0 - out.tex_coords.y;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    var color: vec3f;

    switch debug.channel {
        case 0u: {
            color = textureSample(position_texture, texture_sampler, in.tex_coords).xyz;
        }
        case 1u: {
            color = textureSample(normal_texture, texture_sampler, in.tex_coords).xyz;
        }
        case 2u: {
            color = textureSample(diffuse_occlusion_texture, texture_sampler, in.tex_coords).xyz;
        }
        case 3u: {
            color = grayscale(textureSample(diffuse_occlusion_texture, texture_sampler, in.tex_coords).w);
        }
        case 4u: {
            color = textureSample(specular_shininess_texture, texture_sampler, in.tex_coords).xyz;
        }
        case 5u: {
            color = grayscale(textureSample(specular_shininess_texture, texture_sampler, in.tex_coords).w);
        }
        case 6u: {
            color = textureSample(emission_texture, texture_sampler, in.tex_coords).xyz;
        }
        default: {
        }
    }

    out.color = vec4f(color, 1.0);

    return out;
}

fn grayscale(x: f32) -> vec3f {
    return vec3f(x, x, x);
}