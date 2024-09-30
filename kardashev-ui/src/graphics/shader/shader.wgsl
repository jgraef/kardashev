struct CameraUniform {
    view_projection: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct InstanceInput {
    @location(3) transform_0: vec4<f32>,
    @location(4) transform_1: vec4<f32>,
    @location(5) transform_2: vec4<f32>,
    @location(6) transform_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    let transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
    );

    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_projection * transform * vec4<f32>(model.position, 1.0);

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
    return textureSample(material_diffuse_texture_view, material_diffuse_sampler, in.tex_coords);
}
