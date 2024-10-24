use palette::Srgb;

#[derive(Clone, Copy, Debug)]
pub struct AmbientLight {
    pub color: Srgb<f32>,
}

#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub color: Srgb<f32>,
}

impl PointLight {
    pub fn new(color: Srgb<f32>) -> Self {
        Self { color }
    }
}
