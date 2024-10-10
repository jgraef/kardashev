use hecs::Entity;
use nalgebra::Perspective3;
use palette::{
    Srgba,
    WithAlpha,
};

use crate::{
    error::Error,
    world::{
        OneshotSystem,
        RunSystemContext,
    },
};

#[derive(Debug)]
pub struct Camera {
    pub aspect: f32,
    pub fovy: f32,
    pub z_near: f32,
    pub z_far: f32,

    pub projection_matrix: Perspective3<f32>,
}

impl Camera {
    pub fn new(aspect: f32, fovy: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            aspect,
            fovy,
            z_near,
            z_far,
            projection_matrix: camera_matrix(aspect, fovy, z_near, z_far),
        }
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
        self.recalculate_matrix();
    }

    pub fn recalculate_matrix(&mut self) {
        self.projection_matrix = camera_matrix(self.aspect, self.fovy, self.z_near, self.z_far);
    }
}

fn camera_matrix(aspect: f32, fovy: f32, z_near: f32, z_far: f32) -> Perspective3<f32> {
    Perspective3::new(aspect, fovy, z_near, z_far)
}

#[derive(Clone, Copy, Debug)]
pub struct ClearColor {
    pub clear_color: Srgba<f32>,
}

impl ClearColor {
    pub fn new(clear_color: Srgba<f32>) -> Self {
        Self { clear_color }
    }
}

impl Default for ClearColor {
    fn default() -> Self {
        Self {
            clear_color: palette::named::BLACK.into_format().with_alpha(1.0),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChangeCameraAspectRatio {
    pub camera_entity: Entity,
    pub aspect: f32,
}

impl OneshotSystem for ChangeCameraAspectRatio {
    fn label(&self) -> &'static str {
        "change-camera-aspect-ratio"
    }

    async fn run<'c: 'd, 'd>(self, context: &'d mut RunSystemContext<'c>) -> Result<(), Error> {
        let mut camera = context
            .world
            .get::<&mut Camera>(self.camera_entity)
            .unwrap();
        camera.set_aspect(self.aspect);
        Ok(())
    }
}
