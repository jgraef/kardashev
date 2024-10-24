#import camera.wgsl::Camera;
#import light.wgsl::Lights;
#import render_3d.wgsl::{VertexInput, InstanceInput, vs_main_inner};

@group(1) @binding(0)
var<uniform> camera: Camera;

@group(2) @binding(0)
var<uniform> light: Lights;

struct InstanceInput {
    @location(5) model_transform_a: vec4<f32>,
    @location(6) model_transform_b: vec4<f32>,
    @location(7) model_transform_c: vec4<f32>,
    @location(8) model_transform_d: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) world_normal: vec3<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@group(0) @binding(0)
var material_albedo_texture_view: texture_2d<f32>;
@group(0) @binding(1)
var material_albedo_sampler: sampler;
@group(0) @binding(2)
var material_normal_texture_view: texture_2d<f32>;
@group(0) @binding(3)
var material_normal_sampler: sampler;
@group(0) @binding(4)
var material_metalness_texture_view: texture_2d<f32>;
@group(0) @binding(5)
var material_metalness_sampler: sampler;
@group(0) @binding(6)
var material_roughness_texture_view: texture_2d<f32>;
@group(0) @binding(7)
var material_roughness_sampler: sampler;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_transform = mat4x4<f32>(
        instance.model_transform_a,
        instance.model_transform_b,
        instance.model_transform_c,
        instance.model_transform_d,
    );
    let inner = vs_main_inner(model, model_transform, camera);
    var out: VertexOutput;
    out.clip_position = inner.clip_position;
    out.tex_coords = inner.tex_coords;
    out.world_position = inner.world_position;
    out.world_normal = inner.world_normal;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    // todo
    var out: FragmentOutput;
    out.color = vec4<f32>(1.0, 0.0, 1.0, 1.0);
    return out;
}
