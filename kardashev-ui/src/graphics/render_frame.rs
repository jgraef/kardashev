use std::{
    fmt::Debug,
    task::Poll,
};

use crate::{
    graphics::{
        camera::RenderTarget,
        Backend,
        Error,
        Surface,
        SurfaceSize,
    },
    world::{
        resource::Resources,
        system::{
            System,
            SystemContext,
        },
        Label,
    },
};

#[derive(Debug, Default)]
pub struct RenderingSystem;

impl System for RenderingSystem {
    type Error = Error;

    fn label(&self) -> &'static str {
        "rendering"
    }

    fn poll_system(
        &mut self,
        _task_context: &mut std::task::Context<'_>,
        system_context: &mut SystemContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let mut render_targets = system_context
            .world
            .query::<(&mut RenderTarget, Option<&Label>)>();

        for (render_target_entity, (render_target, label)) in render_targets.iter() {
            let render_target = render_target.inner.get_mut();

            if let Some(surface_size) = render_target.surface_size_listener.poll() {
                tracing::debug!(?label, ?surface_size, "surface resized");
                render_target
                    .render_pass
                    .resize(&render_target.backend, surface_size);
            }

            if !render_target.is_visible() {
                tracing::trace!(?label, "skipping render target (not visible)");
                continue;
            }

            tracing::trace!(?label, "rendering render target");

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

        Poll::Ready(Ok(()))
    }
}

pub trait RenderPass: Debug {
    fn render(&mut self, render_pass_context: &mut RenderPassContext);

    fn resize(&mut self, backend: &Backend, surface_size: SurfaceSize);
}

pub trait CreateRenderPass: RenderPass {
    fn create_render_pass(surface: &Surface) -> Self;
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
