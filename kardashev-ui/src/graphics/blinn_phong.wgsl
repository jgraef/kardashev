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


@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    return vs_main_inner(model, instance, camera);
}


@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    // todo: use the other material textures, if applicable

    let view_direction = normalize(camera.view_position - in.world_position);
    
    let ambient_texture_color = textureSample(material_ambient_texture_view, material_ambient_sampler, in.tex_coords);
    let ambient_color = light.ambient_light * ambient_texture_color;

    var diffuse_color = vec4f(0.0);
    let diffuse_texture_color = textureSample(material_diffuse_texture_view, material_diffuse_sampler, in.tex_coords);

    var specular_color = vec4f(0.0);
    let specular_texture_color = textureSample(material_specular_texture_view, material_specular_sampler, in.tex_coords);
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
    diffuse_color *= diffuse_texture_color;
    specular_color *= specular_texture_color;
    
    out.color = ambient_color + diffuse_color + specular_color;
    out.color.w = 1.0;

    return out;
}
