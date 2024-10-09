struct CameraUniform {
    view_projection: mat4x4<f32>,
};

struct LightUniform {
    ambient_color: vec4<f32>,
    diffuse_color: vec4<f32>,
    position: vec3<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> light: LightUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct InstanceInput {
    @location(3) model_transform_0: vec4<f32>,
    @location(4) model_transform_1: vec4<f32>,
    @location(5) model_transform_2: vec4<f32>,
    @location(6) model_transform_3: vec4<f32>,
    //@location(7) normal_1: vec3<f32>,
    //@location(8) normal_2: vec3<f32>,
    //@location(9) normal_3: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) world_normal: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    let model_transform = mat4x4<f32>(
        instance.model_transform_0,
        instance.model_transform_1,
        instance.model_transform_2,
        instance.model_transform_3,
    );

    /*let normal_matrix = mat3x3<f32>(
        instance.normal_0,
        instance.normal_1,
        instance.normal_2,
    );*/

    out.tex_coords = model.tex_coords;

    let world_position = model_transform * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;

    out.world_normal = normalize((model_transform * vec4<f32>(model.normal, 0.0)).xyz);

    out.clip_position = camera.view_projection * world_position;

    return out;
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


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let diffuse_texture_color = textureSample(material_diffuse_texture_view, material_diffuse_sampler, in.tex_coords);

    let ambient_color = light.ambient_color.xyz * light.ambient_color.w;

    let light_direction = normalize(light.position - in.world_position);
    let diffuse_strength = max(dot(in.world_normal, light_direction), 0.0);
    let diffuse_color = light.diffuse_color.xyz * light.diffuse_color.w * diffuse_strength;

    let color_rgb = (ambient_color + diffuse_color) * diffuse_texture_color.xyz;
    let color_rgba = vec4<f32>(color_rgb, diffuse_texture_color.w);
    //let color = diffuse_texture_color;

    return color_rgba;
}
