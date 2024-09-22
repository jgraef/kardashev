use nalgebra::{
    Projective3,
};
use palette::Srgba;

pub struct Camera {
    pub clear_color: Option<Srgba<f32>>,
    pub projection: Projective3<f32>,
}
