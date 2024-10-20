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
        render_frame::{
            CreateRenderPass,
            CreateRenderPassContext,
            DynRenderPass,
            RenderPass,
        },
        Surface,
        SurfaceSize,
        SurfaceSizeListener,
        SurfaceVisibilityListener,
    },
    utils::thread_local_cell::ThreadLocalCell,
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

#[derive(Debug)]
pub struct RenderTarget {
    pub(super) inner: ThreadLocalCell<RenderTargetInner>,
}

impl RenderTarget {
    pub fn new<P: CreateRenderPass + 'static>(surface: &Surface, create_render_pass: P) -> Self {
        Self {
            inner: ThreadLocalCell::new(RenderTargetInner {
                surface: surface.surface.clone(),
                surface_configuration: surface.surface_configuration.clone(),
                surface_size_listener: surface.size_listener(),
                surface_visibility_listener: surface.visibility_listener(),
                backend: surface.backend.clone(),
                render_pass: DynRenderPass::new(create_render_pass.create_render_pass(
                    &CreateRenderPassContext {
                        backend: &surface.backend,
                        surface_size: surface.size(),
                        surface_format: surface.format(),
                    },
                )),
            }),
        }
    }
}

#[derive(Debug)]
pub(super) struct RenderTargetInner {
    pub surface: Arc<wgpu::Surface<'static>>,
    pub surface_configuration: wgpu::SurfaceConfiguration,
    pub surface_size_listener: SurfaceSizeListener,
    pub surface_visibility_listener: SurfaceVisibilityListener,
    pub backend: Backend,
    pub render_pass: DynRenderPass,
}

impl RenderTargetInner {
    pub fn is_visible(&self) -> bool {
        self.surface_visibility_listener.is_visible()
    }

    pub fn resize(&mut self, surface_size: SurfaceSize) {
        self.surface_configuration.width = surface_size.width;
        self.surface_configuration.height = surface_size.height;
        self.surface
            .configure(&self.backend.device, &self.surface_configuration);

        self.render_pass.resize(&self.backend, surface_size);
    }
}
