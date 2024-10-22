const MAX_SPOT_LIGHTS: u32 = 16;

struct Lights {
    ambient_light: vec4<f32>,
    num_point_lights: u32,
    point_lights: array<SpotLight, MAX_SPOT_LIGHTS>,
};

struct SpotLight {
    position: vec4f,
    color: vec4f,
}
