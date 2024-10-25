#import shared.wgsl::{Globals, MAX_SPOT_LIGHTS};

struct VertexInput {
    @builtin(vertex_index)
    vertex_index: u32,
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

@group(1) @binding(0)
var texture_sampler: sampler;
@group(1) @binding(1)
var position_texture: texture_2d<f32>;
@group(1) @binding(2)
var normal_texture: texture_2d<f32>;
@group(1) @binding(3)
var diffuse_texture: texture_2d<f32>;
@group(1) @binding(4)
var specular_texture: texture_2d<f32>;

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
    
    let position = textureSample(position_texture, texture_sampler, in.tex_coords);
    let normal = textureSample(normal_texture, texture_sampler, in.tex_coords);
    let diffuse = textureSample(diffuse_texture, texture_sampler, in.tex_coords);
    let specular = textureSample(specular_texture, texture_sampler, in.tex_coords);

    let view_direction = normalize(position.xyz - globals.view_position);
    
    out.color = vec4f(0.0, 0.0, 0.0, 1.0);

    return out;
}
