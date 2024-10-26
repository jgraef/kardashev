use bytemuck::{
    Pod,
    Zeroable,
};
use nalgebra::Point3;
use palette::Srgb;

use crate::graphics::{
    camera::CameraProjection,
    transform::GlobalTransform,
    utils::Srgb32Ext,
};

pub const MAX_POINT_LIGHTS: usize = 16;

#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
pub struct GlobalsUniform {
    pub view_projection: [f32; 16],
    pub view_position: [f32; 3],
    pub aspect: f32,
    pub ambient_light: [f32; 3],
    pub num_point_lights: u32,
    pub point_lights: [PointLightUniform; MAX_POINT_LIGHTS],
}

impl GlobalsUniform {
    pub fn set_camera(&mut self, camera: &CameraProjection, transform: &GlobalTransform) {
        self.view_projection = (camera.projection_matrix.as_matrix()
            * transform.model_matrix.inverse().to_homogeneous())
        .as_slice()
        .try_into()
        .unwrap();
        self.view_position = transform
            .model_matrix
            .isometry
            .translation
            .vector
            .as_slice()
            .try_into()
            .unwrap();
        self.aspect = camera.projection_matrix.aspect();
    }

    pub fn set_ambient_color(&mut self, color: Srgb<f32>) {
        self.ambient_light = color.as_array3();
    }

    pub fn add_point_light(&mut self, position: Point3<f32>, color: Srgb<f32>) -> bool {
        let index: usize = self.num_point_lights.try_into().unwrap();
        if index < MAX_POINT_LIGHTS {
            self.point_lights[index] = PointLightUniform::new(position, color);
            self.num_point_lights += 1;
            true
        }
        else {
            false
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
pub struct PointLightUniform {
    pub position: [f32; 3],
    _padding0: u32,
    pub color: [f32; 3],
    _padding1: u32,
}

impl PointLightUniform {
    pub fn new(position: Point3<f32>, color: Srgb<f32>) -> Self {
        Self {
            position: position.coords.as_slice().try_into().unwrap(),
            _padding0: 0,
            color: color.as_array3(),
            _padding1: 0,
        }
    }
}
