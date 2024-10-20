use std::fmt::Debug;

use crate::{
    ecs::{
        resource::Resources,
        system::{
            System,
            SystemContext,
        },
        Label,
    },
    graphics::{
        camera::RenderTarget,
        Backend,
        Error,
        SurfaceSize,
    },
};

#[derive(Debug, Default)]
pub struct RenderingSystem;

impl System for RenderingSystem {
    type Error = Error;

    fn label(&self) -> &'static str {
        "rendering"
    }

    fn poll_system(&mut self, system_context: &mut SystemContext<'_>) -> Result<(), Self::Error> {
        let mut render_targets = system_context
            .world
            .query::<(&mut RenderTarget, Option<&Label>)>();

        for (render_target_entity, (render_target, label)) in render_targets.iter() {
            let render_target = render_target.inner.get_mut();

            if let Some(surface_size) = render_target.surface_size_listener.poll() {
                tracing::debug!(?label, ?surface_size, "surface resized");
                render_target.resize(surface_size);
            }

            if !render_target.is_visible() {
                tracing::debug!(?label, "skipping render target (not visible)");
                continue;
            }

            tracing::trace!(?label, "rendering frame");

            let target_texture = render_target
                .surface
                .get_current_texture()
                .expect("could not get target texture");

            let target_view = target_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = render_target.backend.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("render encoder"),
                },
            );

            render_target.render_pass.render(&mut RenderPassContext {
                backend: &render_target.backend,
                encoder: &mut encoder,
                target_view: &target_view,
                render_target_entity,
                world: &system_context.world,
                resources: &mut system_context.resources,
            });

            render_target.backend.queue.submit([encoder.finish()]);
            target_texture.present();
        }

        Ok(())
    }
}

pub trait CreateRenderPass {
    type RenderPass: RenderPass;

    fn create_render_pass(self, context: &CreateRenderPassContext) -> Self::RenderPass;
}

#[derive(Clone, Copy, Debug)]
pub struct CreateRenderPassContext<'a> {
    pub backend: &'a Backend,
    pub surface_size: SurfaceSize,
    pub surface_format: wgpu::TextureFormat,
}

pub trait RenderPass {
    fn render(&mut self, context: &mut RenderPassContext);

    fn resize(&mut self, backend: &Backend, surface_size: SurfaceSize);
}

// todo: impl Debug
pub struct RenderPassContext<'a> {
    pub backend: &'a Backend,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub target_view: &'a wgpu::TextureView,
    pub render_target_entity: hecs::Entity,
    pub world: &'a hecs::World,
    pub resources: &'a mut Resources,
}

pub struct DynRenderPass {
    inner: Box<dyn RenderPass>,
}

impl DynRenderPass {
    pub fn new(render_pass: impl RenderPass + 'static) -> Self {
        DynRenderPass {
            inner: Box::new(render_pass),
        }
    }
}

impl Debug for DynRenderPass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxedRenderPass").finish_non_exhaustive()
    }
}

impl RenderPass for DynRenderPass {
    fn render(&mut self, render_pass_context: &mut RenderPassContext) {
        self.inner.render(render_pass_context);
    }

    fn resize(&mut self, backend: &Backend, surface_size: SurfaceSize) {
        self.inner.resize(backend, surface_size);
    }
}
