const MAX_SPOT_LIGHTS: u32 = 16;

struct Globals {
    view_projection: mat4x4<f32>,
    view_position: vec3<f32>,
    time: f32,
    aspect: f32,
    ambient_light: vec3<f32>,
    num_point_lights: u32,
    point_lights: array<SpotLight, MAX_SPOT_LIGHTS>,
}

struct SpotLight {
    position: vec3f,
    color: vec3f,
}
