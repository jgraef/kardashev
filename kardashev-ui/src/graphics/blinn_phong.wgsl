#import camera.wgsl::Camera;
#import light.wgsl::Lights;
#import render_3d.wgsl::{VertexInput, vs_main_inner};

@group(1) @binding(0)
var<uniform> camera: Camera;

@group(2) @binding(0)
var<uniform> light: Lights;

struct InstanceInput {
    @location(3) model_transform_a: vec4<f32>,
    @location(4) model_transform_b: vec4<f32>,
    @location(5) model_transform_c: vec4<f32>,
    @location(6) model_transform_d: vec4<f32>,
    @location(7) material_ambient_color: vec3<f32>,
    @location(8) material_diffuse_color: vec3<f32>,
    @location(9) material_specular_color: vec3<f32>,
    @location(10) material_emissive_color: vec3<f32>,
    @location(11) material_shininess: f32,
    @location(12) material_dissolve: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) world_normal: vec3<f32>,
    @location(3) material_ambient_color: vec3<f32>,
    @location(4) material_diffuse_color: vec3<f32>,
    @location(5) material_specular_color: vec3<f32>,
    @location(6) material_emissive_color: vec3<f32>,
    @location(7) material_shininess: f32,
    @location(8) material_dissolve: f32,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@group(0) @binding(0)
var material_ambient_texture_view: texture_2d<f32>;
@group(0) @binding(1)
var material_ambient_sampler: sampler;
@group(0) @binding(2)
var material_diffuse_texture_view: texture_2d<f32>;
@group(0) @binding(3)
var material_diffuse_sampler: sampler;
@group(0) @binding(4)
var material_specular_texture_view: texture_2d<f32>;
@group(0) @binding(5)
var material_specular_sampler: sampler;
@group(0) @binding(6)
var material_normal_texture_view: texture_2d<f32>;
@group(0) @binding(7)
var material_normal_sampler: sampler;
@group(0) @binding(8)
var material_shininess_texture_view: texture_2d<f32>;
@group(0) @binding(9)
var material_shininess_sampler: sampler;
@group(0) @binding(10)
var material_dissolve_texture_view: texture_2d<f32>;
@group(0) @binding(11)
var material_dissolve_sampler: sampler;
@group(0) @binding(12)
var material_emissive_texture_view: texture_2d<f32>;
@group(0) @binding(13)
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
    let inner = vs_main_inner(vertex, model_transform, camera);

    var out: VertexOutput;
    out.clip_position = inner.clip_position;
    out.tex_coords = inner.tex_coords;
    out.world_position = inner.world_position;
    out.world_normal = inner.world_normal;
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
    // todo: use the other material textures, if applicable

    let view_direction = normalize(camera.view_position - in.world_position);
    
    let ambient_texture_color = textureSample(material_ambient_texture_view, material_ambient_sampler, in.tex_coords).xyz;
    let ambient_color = light.ambient_light * ambient_texture_color * in.material_ambient_color;

    let emissive_texture_color = textureSample(material_emissive_texture_view, material_emissive_sampler, in.tex_coords).xyz;
    let emissive_color = emissive_texture_color * in.material_emissive_color;

    var diffuse_color = vec3f(0.0);
    let diffuse_texture_color = textureSample(material_diffuse_texture_view, material_diffuse_sampler, in.tex_coords).xyz;

    var specular_color = vec3f(0.0);
    let specular_texture_color = textureSample(material_specular_texture_view, material_specular_sampler, in.tex_coords).xyz;
    //let shininess = textureSample(material_shininess_texture_view, material_shininess_sampler, in.tex_coords).x;
    let shininess = 32.0;

    // spot lights
    for (var i: u32 = 0; i < light.num_point_lights; i++) {
        let light_direction = normalize(light.point_lights[i].position.xyz - in.world_position);
        let reflect_direction = reflect(-light_direction, in.world_normal);
        
        let diffuse_strength = max(dot(in.world_normal, light_direction), 0.0);
        diffuse_color += light.point_lights[i].color * diffuse_strength;

        let specular_strength = pow(max(dot(view_direction, reflect_direction), 0.0), shininess);
        specular_color += light.point_lights[i].color * specular_strength;
    }
    diffuse_color *= diffuse_texture_color * in.material_diffuse_color;
    specular_color *= specular_texture_color * in.material_specular_color;
    
    out.color = vec4f(ambient_color + emissive_color + diffuse_color + specular_color, 1.0);

    return out;
}
