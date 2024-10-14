use std::{
    sync::Arc,
    task::Poll,
};

use hecs::Entity;
use nalgebra::Perspective3;
use palette::{
    Srgba,
    WithAlpha,
};

use crate::{
    error::Error,
    graphics::{
        backend::Backend,
        render_frame::{
            CreateRenderPass,
            RenderPass,
        },
        Surface,
        SurfaceSizeListener,
        SurfaceVisibilityListener,
    },
    utils::thread_local_cell::ThreadLocalCell,
    world::system::{
        System,
        SystemContext,
    },
};

#[derive(Debug)]
pub struct CameraProjection {
    pub aspect: f32,
    pub fovy: f32,
    pub z_near: f32,
    pub z_far: f32,

    pub projection_matrix: Perspective3<f32>,
}

impl CameraProjection {
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

impl System for ChangeCameraAspectRatio {
    type Error = Error;

    fn label(&self) -> &'static str {
        "change-camera-aspect-ratio"
    }

    fn poll_system(
        &mut self,
        _task_context: &mut std::task::Context<'_>,
        system_context: &mut SystemContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let mut camera = system_context
            .world
            .get::<&mut CameraProjection>(self.camera_entity)
            .unwrap();
        camera.set_aspect(self.aspect);
        Poll::Ready(Ok(()))
    }
}

#[derive(Debug)]
pub struct RenderTarget {
    pub(super) inner: ThreadLocalCell<RenderTargetInner>,
}

impl RenderTarget {
    pub fn from_surface<R: CreateRenderPass + 'static>(surface: &Surface) -> Self {
        Self {
            inner: ThreadLocalCell::new(RenderTargetInner {
                surface: surface.surface.clone(),
                surface_size_listener: surface.size_listener(),
                surface_visibility_listener: surface.visibility_listener(),
                backend: surface.backend.clone(),
                render_pass: Box::new(R::create_render_pass(surface)),
            }),
        }
    }
}

#[derive(Debug)]
pub(super) struct RenderTargetInner {
    pub surface: Arc<wgpu::Surface<'static>>,
    pub surface_size_listener: SurfaceSizeListener,
    pub surface_visibility_listener: SurfaceVisibilityListener,
    pub backend: Backend,
    pub render_pass: Box<dyn RenderPass>,
}

impl RenderTargetInner {
    pub fn is_visible(&self) -> bool {
        self.surface_visibility_listener.is_visible()
    }
}
