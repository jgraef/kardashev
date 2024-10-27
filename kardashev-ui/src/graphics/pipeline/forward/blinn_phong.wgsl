#import ../globals.wgsl::{Globals, PointLight, MAX_POINT_LIGHTS};

@group(0) @binding(0)
var<uniform> globals: Globals;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
    @location(2) normal: vec3f,
    @location(3) tangent: vec3f,
    @location(4) bitangent: vec3f,
}

struct InstanceInput {
    @location(5) model_transform_x: vec4f,
    @location(6) model_transform_y: vec4f,
    @location(7) model_transform_z: vec4f,
    @location(8) model_transform_w: vec4f,
    @location(9) material_ambient_color: vec3f,
    @location(10) material_diffuse_color: vec3f,
    @location(11) material_specular_color: vec3f,
    @location(12) material_emissive_color: vec3f,
    @location(13) material_shininess: f32,
    @location(14) material_dissolve: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
    @location(1) tangent_position: vec3f,
    @location(2) tangent_view_position: vec3f,
    @location(3) tangent_matrix_x: vec3f,
    @location(4) tangent_matrix_y: vec3f,
    @location(5) tangent_matrix_z: vec3f,
    @location(6) material_ambient_color: vec3f,
    @location(7) material_diffuse_color: vec3f,
    @location(8) material_specular_color: vec3f,
    @location(9) material_emissive_color: vec3f,
    @location(10) material_shininess: f32,
    @location(11) material_dissolve: f32,
}

struct FragmentOutput {
    @location(0) color: vec4f,
}

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
        instance.model_transform_x,
        instance.model_transform_y,
        instance.model_transform_z,
        instance.model_transform_w,
    );

    var out: VertexOutput;

    let world_position = model_transform * vec4f(vertex.position, 1.0);

    // this works if the model_transform uses uniform scaling.
    let world_normal = normalize((model_transform * vec4f(vertex.normal, 0.0)).xyz);
    let world_tangent = normalize((model_transform * vec4f(vertex.tangent, 0.0)).xyz);
    let world_bitangent = normalize((model_transform * vec4f(vertex.bitangent, 0.0)).xyz);
    //let world_normal = vertex.normal;
    //let world_tangent = vertex.tangent;
    //let world_bitangent = vertex.bitangent;

    let tangent_matrix = transpose(mat3x3<f32>(
        world_tangent,
        world_bitangent,
        world_normal,
    ));

    out.clip_position = globals.view_projection * world_position;
    out.tex_coords = vertex.tex_coords;
        
    out.tangent_position = tangent_matrix * world_position.xyz;
    out.tangent_view_position = tangent_matrix * globals.view_position.xyz;
    out.tangent_matrix_x = tangent_matrix.x;
    out.tangent_matrix_y = tangent_matrix.y;
    out.tangent_matrix_z = tangent_matrix.z;
    
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
    
    let tangent_matrix = mat3x3f(
        in.tangent_matrix_x,
        in.tangent_matrix_y,
        in.tangent_matrix_z,
    );

    let tangent_normal = textureSample(material_normal_texture_view, material_normal_sampler, in.tex_coords).xyz * 2.0 - 1.0;
    //let tangent_normal = vec3f(0.0, 0.0, 1.0);

    //let view_direction = normalize(globals.view_position - in.world_position);
    let view_direction = normalize(in.tangent_view_position - in.tangent_position);
    
    let ambient_texture_color = textureSample(material_ambient_texture_view, material_ambient_sampler, in.tex_coords).xyz;
    let ambient_color = globals.ambient_light * ambient_texture_color * in.material_ambient_color;

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
    for (var i: u32 = 0; i < globals.num_point_lights; i++) {
        let light_position = tangent_matrix * globals.point_lights[i].position;
        let light_direction = normalize(light_position - in.tangent_position);
        
        let reflect_direction = reflect(-light_direction, tangent_normal);
        //let half_direction = normalize(view_direction + light_direction);
        
        let diffuse_strength = max(dot(tangent_normal, light_direction), 0.0);
        diffuse_color += globals.point_lights[i].color * diffuse_strength;

        let specular_strength = pow(max(dot(view_direction, reflect_direction), 0.0), shininess);
        //let specular_strength = pow(max(dot(tangent_normal, half_direction), 0.0), shininess);
        specular_color += globals.point_lights[i].color * specular_strength;
    }
    diffuse_color *= diffuse_texture_color * in.material_diffuse_color;
    specular_color *= specular_texture_color * in.material_specular_color;
    
    out.color = vec4f(ambient_color + emissive_color + diffuse_color + specular_color, 1.0);

    return out;
}
