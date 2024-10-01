use nalgebra::Perspective3;
use palette::Srgba;

#[derive(Debug)]
pub struct Camera {
    pub aspect: f32,
    pub fovy: f32,
    pub z_near: f32,
    pub z_far: f32,

    pub matrix: Perspective3<f32>,
}

impl Camera {
    pub fn new(aspect: f32, fovy: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            aspect,
            fovy,
            z_near,
            z_far,
            matrix: Perspective3::new(aspect, fovy, z_near, z_far),
        }
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
        self.recalculate_matrix();
    }

    pub fn recalculate_matrix(&mut self) {
        self.matrix = Perspective3::new(self.aspect, self.fovy, self.z_near, self.z_far);
    }
}

#[derive(Debug)]
pub struct ClearColor {
    pub clear_color: Srgba<f32>,
}
