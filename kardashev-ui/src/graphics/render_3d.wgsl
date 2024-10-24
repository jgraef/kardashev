#import camera.wgsl::Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
};

struct VertexOutput {
    clip_position: vec4<f32>,
    tex_coords: vec2<f32>,
    world_position: vec3<f32>,
    world_normal: vec3<f32>,
}

fn vs_main_inner(
    model: VertexInput,
    model_transform: mat4x4<f32>,
    camera: Camera,
) -> VertexOutput {
    var out: VertexOutput;
    
    out.tex_coords = model.tex_coords;

    let world_position = model_transform * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;

    // this works if the model_transform uses uniform scaling.
    out.world_normal = normalize((model_transform * vec4<f32>(model.normal, 0.0)).xyz);

    out.clip_position = camera.view_projection * world_position;

    return out;
}
