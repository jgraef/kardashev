#import ../globals.wgsl::{Globals, MAX_SPOT_LIGHTS};

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
    
    let world_position = textureSample(position_texture, texture_sampler, in.tex_coords).xyz;
    let world_normal = textureSample(normal_texture, texture_sampler, in.tex_coords).xyz;
    let diffuse_occlusion = textureSample(diffuse_occlusion_texture, texture_sampler, in.tex_coords);
    let specular_shininess = textureSample(specular_shininess_texture, texture_sampler, in.tex_coords);
    let emission = textureSample(emission_texture, texture_sampler, in.tex_coords);

    let diffuse_texture_color = diffuse_occlusion.xyz;
    let specular_texture_color = specular_shininess.xyz;
    let shininess = specular_shininess.w;
    
    let view_direction = normalize(globals.view_position - world_position);
    
    let emission_color = emission.xyz;
    let ambient_color = globals.ambient_light * diffuse_occlusion.w;
    var diffuse_color = vec3f(0.0);
    var specular_color = vec3f(0.0);

    // point lights
    for (var i: u32 = 0; i < globals.num_point_lights; i++) {
        let light_direction = normalize(globals.point_lights[i].position - world_position);
        
        let reflect_direction = reflect(-light_direction, world_normal);
        //let half_direction = normalize(view_direction + light_direction);
        
        let diffuse_strength = max(dot(world_normal, light_direction), 0.0);
        diffuse_color += globals.point_lights[i].color * diffuse_strength;

        let specular_strength = pow(max(dot(view_direction, reflect_direction), 0.0), shininess);
        //let specular_strength = pow(max(dot(tangent_normal, half_direction), 0.0), shininess);
        specular_color += globals.point_lights[i].color * specular_strength;
    }
    diffuse_color *= diffuse_texture_color;
    specular_color *= specular_texture_color;
    
    out.color = vec4f(emission_color + ambient_color + diffuse_color + specular_color, 1.0);

    return out;
}
