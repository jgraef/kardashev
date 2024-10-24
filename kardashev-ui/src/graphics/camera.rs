use std::{
    fmt::Debug,
    sync::Arc,
};

use nalgebra::Perspective3;
use palette::{
    Srgba,
    WithAlpha,
};

use crate::{
    graphics::{
        backend::Backend,
        Surface,
    },
    utils::thread_local_cell::ThreadLocalCell,
};

#[derive(Debug)]
pub struct CameraProjection {
    pub projection_matrix: Perspective3<f32>,
}

impl CameraProjection {
    pub fn new(aspect: f32, fovy: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            projection_matrix:  Perspective3::new(aspect, fovy, z_near, z_far),
        }
    }
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

#[derive(Debug)]
pub struct RenderTarget {
    pub(super) inner: ThreadLocalCell<RenderTargetInner>,
}

impl RenderTarget {
    pub fn from_surface(surface: &Surface) -> Self {
        Self {
            inner: ThreadLocalCell::new(RenderTargetInner::Surface {
                backend: surface.backend.clone(),
                surface: surface.surface.clone(),
            }),
        }
    }

    pub fn from_texture(backend: Backend, texture: Arc<wgpu::Texture>) -> Self {
        Self {
            inner: ThreadLocalCell::new(RenderTargetInner::Texture { backend, texture }),
        }
    }
}

#[derive(Debug)]
pub(super) enum RenderTargetInner {
    Surface {
        backend: Backend,
        surface: Arc<wgpu::Surface<'static>>,
    },
    Texture {
        backend: Backend,
        texture: Arc<wgpu::Texture>,
    },
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DontRender;
