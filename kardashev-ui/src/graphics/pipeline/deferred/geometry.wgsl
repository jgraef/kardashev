#import ../globals.wgsl::{Globals, MAX_SPOT_LIGHTS};

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
    @location(2) normal: vec3f,
    @location(3) tangent: vec3f,
    @location(4) bitangent: vec3f,
}

struct InstanceInput {
    @location(5) model_transform_a: vec4f,
    @location(6) model_transform_b: vec4f,
    @location(7) model_transform_c: vec4f,
    @location(8) model_transform_d: vec4f,
    @location(9) material_ambient_color: vec3f,
    @location(10) material_diffuse_color: vec3f,
    @location(11) material_specular_color: vec3f,
    @location(12) material_emissive_color: vec3f,
    @location(13) material_shininess: f32,
    @location(14) material_dissolve: f32,
}

// todo: are binormals needed?
struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
    @location(2) normal: vec3f,
    @location(3) material_ambient_color: vec3f,
    @location(4) material_diffuse_color: vec3f,
    @location(5) material_specular_color: vec3f,
    @location(6) material_emissive_color: vec3f,
    @location(7) material_shininess: f32,
    @location(8) material_dissolve: f32,
}

struct FragmentOutput {
    @location(0) position: vec4f,
    @location(1) normal: vec4f,
    @location(2) diffuse_occlusion: vec4f,
    @location(3) specular_shininess: vec4f,
    @location(4) emission: vec4f,
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

    let world_position = model_transform * vec4f(vertex.position, 1.0);

    out.clip_position = globals.view_projection * world_position;
    out.position = world_position.xyz;
    out.tex_coords = vertex.tex_coords;
    out.normal = (model_transform * vec4f(vertex.normal, 0.0)).xyz;
    // todo: add contribution from normal texture
        
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

    let diffuse = textureSample(material_diffuse_texture_view, material_diffuse_sampler, in.tex_coords).xyz * in.material_diffuse_color;
    let occlusion = textureSample(material_ambient_texture_view, material_ambient_sampler, in.tex_coords).x * in.material_ambient_color.x;
    let specular = textureSample(material_specular_texture_view, material_specular_sampler, in.tex_coords).xyz * in.material_specular_color;
    let shininess = textureSample(material_shininess_texture_view, material_shininess_sampler, in.tex_coords).x * in.material_shininess;
    let emission = textureSample(material_emissive_texture_view, material_emissive_sampler, in.tex_coords).xyz * in.material_emissive_color;

    out.position = vec4f(in.position, 0.0);
    out.normal = vec4f(in.normal, 0.0);
    out.diffuse_occlusion = vec4f(
        diffuse,
        occlusion,
    );
    out.specular_shininess = vec4f(
        specular,
        shininess,
    );
    out.emission = vec4f(
        emission,
        0.0,
    );

    return out;
}
