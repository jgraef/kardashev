use nalgebra::Projective3;
use palette::Srgba;

#[derive(Debug)]
pub struct Camera {
    pub projection: Projective3<f32>,
}

#[derive(Debug)]
pub struct ClearColor {
    pub clear_color: Srgba<f32>,
}
