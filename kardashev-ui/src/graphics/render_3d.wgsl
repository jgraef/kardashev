#import camera.wgsl::Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct InstanceInput {
    @location(3) model_transform_a: vec4<f32>,
    @location(4) model_transform_b: vec4<f32>,
    @location(5) model_transform_c: vec4<f32>,
    @location(6) model_transform_d: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) world_normal: vec3<f32>,
}

fn vs_main_inner(
    model: VertexInput,
    instance: InstanceInput,
    camera: Camera,
) -> VertexOutput {
    var out: VertexOutput;
    
    let model_transform = mat4x4<f32>(
        instance.model_transform_a,
        instance.model_transform_b,
        instance.model_transform_c,
        instance.model_transform_d,
    );

    out.tex_coords = model.tex_coords;

    let world_position = model_transform * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;

    // this works if the model_transform uses uniform scaling.
    out.world_normal = normalize((model_transform * vec4<f32>(model.normal, 0.0)).xyz);

    out.clip_position = camera.view_projection * world_position;

    return out;
}
