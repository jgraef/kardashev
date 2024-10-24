#import camera.wgsl::Camera;
#import light.wgsl::Lights;

@group(1) @binding(0)
var<uniform> camera: Camera;

@group(2) @binding(0)
var<uniform> light: Lights;

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
    @location(1) tangent_position: vec3<f32>,
    @location(2) tangent_view_position: vec3<f32>,
    @location(3) tangent_light_position: vec3<f32>,
    @location(4) material_ambient_color: vec3<f32>,
    @location(5) material_diffuse_color: vec3<f32>,
    @location(6) material_specular_color: vec3<f32>,
    @location(7) material_emissive_color: vec3<f32>,
    @location(8) material_shininess: f32,
    @location(9) material_dissolve: f32,
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

    var out: VertexOutput;

    let world_position = model_transform * vec4<f32>(vertex.position, 1.0);

    // this works if the model_transform uses uniform scaling.
    let world_normal = normalize((model_transform * vec4<f32>(vertex.normal, 0.0)).xyz);
    let world_tangent = normalize((model_transform * vec4<f32>(vertex.tangent, 0.0)).xyz);
    let world_bitangent = normalize((model_transform * vec4<f32>(vertex.bitangent, 0.0)).xyz);
    //let world_normal = vertex.normal;
    //let world_tangent = vertex.tangent;
    //let world_bitangent = vertex.bitangent;

    let tangent_matrix = transpose(mat3x3<f32>(
        world_tangent,
        world_bitangent,
        world_normal,
    ));

    out.clip_position = camera.view_projection * world_position;
    out.tex_coords = vertex.tex_coords;
        
    out.tangent_position = tangent_matrix * world_position.xyz;
    out.tangent_view_position = tangent_matrix * camera.view_position.xyz;
    // fixme
    //for (var i: u32 = 0; i < light.num_point_lights; i++) {
    //    out.tangent_light_position[i] = tangent_matrix * light.point_lights[i].position;
    //}
    out.tangent_light_position = tangent_matrix * light.point_lights[0].position.xyz;
    
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
    
    let tangent_normal = textureSample(material_normal_texture_view, material_normal_sampler, in.tex_coords).xyz * 2.0 - 1.0;
    //let tangent_normal = vec3f(0.0, 0.0, 1.0);

    //let view_direction = normalize(camera.view_position - in.world_position);
    let view_direction = normalize(in.tangent_view_position - in.tangent_position);
    
    let ambient_texture_color = textureSample(material_ambient_texture_view, material_ambient_sampler, in.tex_coords).xyz;
    let ambient_color = light.ambient_light * ambient_texture_color * in.material_ambient_color;

    let emissive_texture_color = textureSample(material_emissive_texture_view, material_emissive_sampler, in.tex_coords).xyz;
    let emissive_color = emissive_texture_color * in.material_emissive_color;

    var diffuse_color = vec3f(0.0);
    let diffuse_texture_color = textureSample(material_diffuse_texture_view, material_diffuse_sampler, in.tex_coords).xyz;

    var specular_color = vec3f(0.0);
    let specular_texture_color = textureSample(material_specular_texture_view, material_specular_sampler, in.tex_coords).xyz;
    // fixme
    let texture_shininess = textureSample(material_shininess_texture_view, material_shininess_sampler, in.tex_coords).x;
    let shininess = texture_shininess * in.material_shininess;

    // spot lights
    for (var i: u32 = 0; i < light.num_point_lights; i++) {
        let light_direction = normalize(in.tangent_light_position - in.tangent_position);
        
        let reflect_direction = reflect(-light_direction, tangent_normal);
        //let half_direction = normalize(view_direction + light_direction);
        
        let diffuse_strength = max(dot(tangent_normal, light_direction), 0.0);
        diffuse_color += light.point_lights[i].color * diffuse_strength;

        let specular_strength = pow(max(dot(view_direction, reflect_direction), 0.0), shininess);
        //let specular_strength = pow(max(dot(tangent_normal, half_direction), 0.0), shininess);
        specular_color += light.point_lights[i].color * specular_strength;
    }
    diffuse_color *= diffuse_texture_color * in.material_diffuse_color;
    specular_color *= specular_texture_color * in.material_specular_color;
    
    out.color = vec4f(ambient_color + emissive_color + diffuse_color + specular_color, 1.0);

    return out;
}
