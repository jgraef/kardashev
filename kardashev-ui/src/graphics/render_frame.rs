use std::fmt::Debug;

use crate::{
    ecs::{
        resource::Resources,
        system::SystemContext,
        Label,
    },
    graphics::{
        camera::{
            DontRender,
            RenderTarget,
            RenderTargetInner,
        },
        Backend,
        Surface,
        SurfaceSize,
    },
    utils::thread_local_cell::ThreadLocalCell,
};

pub fn rendering_system(system_context: &mut SystemContext) {
    let mut render_targets = system_context
        .world
        .query::<(&RenderTarget, &mut AttachedRenderPass, Option<&Label>)>()
        .without::<&DontRender>();

    for (render_target_entity, (render_target, render_pass, label)) in render_targets.iter() {
        match render_target.inner.get() {
            RenderTargetInner::Surface { backend, surface } => {
                let surface_texture = surface
                    .get_current_texture()
                    .expect("could not get target texture");
                render_to_texture(
                    backend,
                    render_pass,
                    &surface_texture.texture,
                    render_target_entity,
                    &system_context.world,
                    &mut system_context.resources,
                    label,
                );
                surface_texture.present();
            }
            RenderTargetInner::Texture { backend, texture } => {
                render_to_texture(
                    backend,
                    render_pass,
                    texture,
                    render_target_entity,
                    &system_context.world,
                    &mut system_context.resources,
                    label,
                );
            }
        };
    }
}

fn render_to_texture(
    backend: &Backend,
    render_pass: &mut AttachedRenderPass,
    texture: &wgpu::Texture,
    render_target_entity: hecs::Entity,
    world: &hecs::World,
    resources: &mut Resources,
    label: Option<&Label>,
) {
    tracing::trace!(?label, "rendering frame");
    let target_size = SurfaceSize::from_texture(texture);
    let target_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render encoder"),
        });

    render_pass.render(&mut RenderPassContext {
        backend: &backend,
        encoder: &mut encoder,
        target_view: &target_view,
        target_size,
        render_target_entity,
        world,
        resources,
    });

    backend.queue.submit([encoder.finish()]);
}

pub trait CreateRenderPass: Sized {
    type RenderPass: RenderPass;

    fn create_render_pass(self, context: &CreateRenderPassContext) -> Self::RenderPass;

    fn create_render_pass_from_surface(self, surface: &Surface) -> Self::RenderPass {
        self.create_render_pass(&CreateRenderPassContext::from_surface(surface))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CreateRenderPassContext<'a> {
    pub backend: &'a Backend,
    pub surface_size: SurfaceSize,
    pub surface_format: wgpu::TextureFormat,
}

impl<'a> CreateRenderPassContext<'a> {
    pub fn from_surface(surface: &'a Surface) -> Self {
        Self {
            backend: &surface.backend,
            surface_size: surface.size(),
            surface_format: surface.format(),
        }
    }
}

pub trait RenderPass {
    fn render(&mut self, context: &mut RenderPassContext);
}

// todo: impl Debug
pub struct RenderPassContext<'a> {
    pub backend: &'a Backend,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub target_view: &'a wgpu::TextureView,
    pub target_size: SurfaceSize,
    pub render_target_entity: hecs::Entity,
    pub world: &'a hecs::World,
    pub resources: &'a mut Resources,
}

pub struct AttachedRenderPass {
    inner: ThreadLocalCell<Box<dyn RenderPass>>,
}

impl AttachedRenderPass {
    pub fn new(render_pass: impl RenderPass + 'static) -> Self {
        Self {
            inner: ThreadLocalCell::new(Box::new(render_pass)),
        }
    }
}

impl Debug for AttachedRenderPass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachedRenderPass").finish_non_exhaustive()
    }
}

impl RenderPass for AttachedRenderPass {
    fn render(&mut self, render_pass_context: &mut RenderPassContext) {
        let inner = self.inner.get_mut();
        inner.render(render_pass_context);
    }
}
