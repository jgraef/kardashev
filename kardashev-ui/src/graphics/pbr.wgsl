#import camera.wgsl::Camera;
#import light.wgsl::Lights;
#import render_3d.wgsl::{VertexInput, InstanceInput, VertexOutput, vs_main_inner};

@group(1) @binding(0)
var<uniform> camera: Camera;

@group(2) @binding(0)
var<uniform> light: Lights;

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
    return vs_main_inner(model, instance, camera);
}


@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    // todo
    var out: FragmentOutput;
    out.color = vec4<f32>(1.0, 0.0, 1.0, 1.0);
    return out;
}
