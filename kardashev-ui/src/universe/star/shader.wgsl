struct CameraUniform {
    view_projection: mat4x4f,
    view_position: vec3f,
    time: f32,
    aspect: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(0) model_transform_0: vec4f,
    @location(1) model_transform_1: vec4f,
    @location(2) model_transform_2: vec4f,
    @location(3) model_transform_3: vec4f,
    @location(4) star_color: vec4f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
    @location(1) color: vec4f,
    @location(2) normal: vec3f,
}

struct FragmentOutput {
    @location(0) color: vec4f,
    //@builtin(frag_depth) depth: f32,
}

// quad from 2 triangles
/*const VERTICES = array<vec2f, 6>(
    // 1st triangle
    vec2f(-0.5, -0.5),
    vec2f(-0.5, 0.5),
    vec2f(0.5, 0.5),
    // 2nd triangle
    vec2f(-0.5, -0.5),
    vec2f(0.5, 0.5),
    vec2f(0.5, -0.5),
);*/

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    let model_transform = mat4x4<f32>(
        instance.model_transform_0,
        instance.model_transform_1,
        instance.model_transform_2,
        instance.model_transform_3,
    );

    // can't index the const. see issue[1]
    // [1]: https://github.com/gfx-rs/wgpu/issues/4337
    //let vertex_position = VERTICES[vertex_index];
    var vertices = array<vec2f, 6>(
        // 1st triangle
        vec2f(-1.0, 1.0),
        vec2f(-1.0, -1.0),
        vec2f(1.0, -1.0),
        // 2nd triangle
        vec2f(-1.0, 1.0),
        vec2f(1.0, -1.0),
        vec2f(1.0, 1.0)
    );
    
    let transform = camera.view_projection * model_transform;
    let scale_x = length(transform[0].xyz);
    let scale_y = camera.aspect / scale_x;
    let translation = transform * vec4f(0.0, 0.0, 0.0, 1.0);

    let vertex_position = vertices[vertex_index];
    out.clip_position = translation + vec4f(vertex_position.x * scale_x, vertex_position.y * scale_y, 0.0, 1.0);
    out.tex_coords = vertex_position;
    out.color = instance.star_color;
    //out.normal = normalize((model_transform * vec4f(0.0, 0.0, 1.0, 0.0)).xyz);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    //if dot(in.tex_coords, in.tex_coords) > 1.0 {
    //    discard;
    //}

    var out: FragmentOutput;
    out.color = in.color;
    //out.color = vec4f(1.0, 1.0, 1.0, 1.0);

    return out;
}
