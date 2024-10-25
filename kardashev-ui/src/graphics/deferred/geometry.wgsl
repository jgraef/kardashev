#import shared.wgsl::{Globals, MAX_SPOT_LIGHTS};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}

struct InstanceInput {
    @location(5) model_transform_a: vec4<f32>,
    @location(6) model_transform_b: vec4<f32>,
    @location(7) model_transform_c: vec4<f32>,
    @location(8) model_transform_d: vec4<f32>,
    @location(9) material_ambient_color: vec3<f32>,
    @location(10) material_diffuse_color: vec3<f32>,
    @location(11) material_specular_color: vec3<f32>,
    @location(12) material_emissive_color: vec3<f32>,
    @location(13) material_shininess: f32,
    @location(14) material_dissolve: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    // todo: normals?
    @location(1) material_ambient_color: vec3<f32>,
    @location(2) material_diffuse_color: vec3<f32>,
    @location(3) material_specular_color: vec3<f32>,
    @location(4) material_emissive_color: vec3<f32>,
    @location(5) material_shininess: f32,
    @location(6) material_dissolve: f32,
}

struct FragmentOutput {
    @location(0) position: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) diffuse: vec3<f32>,
    @location(3) specular: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(1) @binding(0)
var material_ambient_texture_view: texture_2d<f32>;
@group(1) @binding(1)
var material_ambient_sampler: sampler;
@group(1) @binding(2)
var material_diffuse_texture_view: texture_2d<f32>;
@group(1) @binding(3)
var material_diffuse_sampler: sampler;
@group(1) @binding(4)
var material_specular_texture_view: texture_2d<f32>;
@group(1) @binding(5)
var material_specular_sampler: sampler;
@group(1) @binding(6)
var material_normal_texture_view: texture_2d<f32>;
@group(1) @binding(7)
var material_normal_sampler: sampler;
@group(1) @binding(8)
var material_shininess_texture_view: texture_2d<f32>;
@group(1) @binding(9)
var material_shininess_sampler: sampler;
@group(1) @binding(10)
var material_dissolve_texture_view: texture_2d<f32>;
@group(1) @binding(11)
var material_dissolve_sampler: sampler;
@group(1) @binding(12)
var material_emissive_texture_view: texture_2d<f32>;
@group(1) @binding(13)
var material_emissive_sampler: sampler;


@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_transform = mat4x4<f32>(
        instance.model_transform_a,
        instance.model_transform_b,
        instance.model_transform_c,
        instance.model_transform_d,
    );

    var out: VertexOutput;

    let world_position = model_transform * vec4<f32>(vertex.position, 1.0);

    out.clip_position = globals.view_projection * world_position;
    out.tex_coords = vertex.tex_coords;
        
    out.material_ambient_color = instance.material_ambient_color;
    out.material_diffuse_color = instance.material_diffuse_color;
    out.material_specular_color = instance.material_specular_color;
    out.material_emissive_color = instance.material_emissive_color;
    out.material_shininess = instance.material_shininess;
    out.material_dissolve = instance.material_dissolve;

    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    return out;
}
