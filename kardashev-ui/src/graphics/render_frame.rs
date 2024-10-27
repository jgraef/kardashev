use std::{
    fmt::Debug,
    time::Duration,
};

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
        pipeline::{
            CreatePipeline,
            CreatePipelineContext,
            RenderPipeline,
            RenderPipelineContext,
            RenderWorldInput,
            TextureConfig,
            TextureOutput,
        },
        Backend,
        Surface,
        SurfaceSize,
    },
    utils::{
        thread_local_cell::ThreadLocalCell,
        time::{
            Instant,
            TicksPerSecond,
        },
    },
};

pub fn rendering_system(system_context: &mut SystemContext) {
    let start_time = Instant::now();

    let mut render_targets = system_context
        .world
        .query::<(&RenderTarget, &mut DynRenderView, Option<&Label>)>()
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

    let end_time = Instant::now();

    let render_frame_info = system_context
        .resources
        .get_mut_or_insert_default::<RenderFrameInfo>();
    render_frame_info.fps.push(end_time);
    render_frame_info.frame_time = end_time.duration_since(start_time);
}

fn render_to_texture(
    backend: &Backend,
    render_pass: &mut DynRenderView,
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

    render_pass.render(&mut RenderViewContext {
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

#[derive(Clone, Debug)]
pub struct RenderFrameInfo {
    fps: TicksPerSecond,
    frame_time: Duration,
}

impl Default for RenderFrameInfo {
    fn default() -> Self {
        Self {
            fps: TicksPerSecond::new(Duration::from_secs(1)),
            frame_time: Duration::default(),
        }
    }
}

impl RenderFrameInfo {
    pub fn fps(&self) -> Option<f32> {
        self.fps.tps()
    }

    pub fn frame_time(&self) -> Duration {
        self.frame_time
    }
}

pub trait CreateRenderView {
    type RenderView;

    fn create_render_view(
        self,
        context: &CreatePipelineContext,
        output_config: &TextureConfig,
    ) -> Self::RenderView;

    fn create_render_view_from_surface(self, surface: &Surface) -> Self::RenderView
    where
        Self: Sized,
    {
        self.create_render_view(
            &CreatePipelineContext {
                backend: &surface.backend,
            },
            &TextureConfig {
                format: surface.format(),
                size: surface.size(),
                usages: wgpu::TextureUsages::RENDER_ATTACHMENT,
            },
        )
    }
}

impl<P> CreateRenderView for P
where
    P: CreatePipeline<InputConfig = (), OutputConfig = TextureConfig>,
{
    type RenderView = P::Pipeline;

    fn create_render_view(
        self,
        context: &CreatePipelineContext,
        output_config: &TextureConfig,
    ) -> Self::RenderView {
        let mut output_config = *output_config;
        CreatePipeline::create_pipeline(self, context, &mut (), &mut output_config)
    }
}

pub trait RenderView {
    fn render(&mut self, context: &mut RenderViewContext);
}

impl<P> RenderView for P
where
    for<'a> P: RenderPipeline<Input<'a> = RenderWorldInput<'a>, Output<'a> = TextureOutput<'a>>,
{
    fn render(&mut self, context: &mut RenderViewContext) {
        RenderPipeline::render(
            self,
            &mut RenderPipelineContext {
                backend: context.backend,
                encoder: context.encoder,
            },
            RenderWorldInput {
                view_entity: context.render_target_entity,
                world: context.world,
                resources: context.resources,
            },
            TextureOutput {
                view: context.target_view,
                size: context.target_size,
            },
        );
    }
}

// todo: impl Debug
pub struct RenderViewContext<'a> {
    pub backend: &'a Backend,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub target_view: &'a wgpu::TextureView,
    pub target_size: SurfaceSize,
    pub render_target_entity: hecs::Entity,
    pub world: &'a hecs::World,
    pub resources: &'a mut Resources,
}

pub struct DynRenderView {
    inner: ThreadLocalCell<Box<dyn RenderView>>,
}

impl DynRenderView {
    pub fn new(render_pass: impl RenderView + 'static) -> Self {
        Self {
            inner: ThreadLocalCell::new(Box::new(render_pass)),
        }
    }
}

impl Debug for DynRenderView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachedRenderPass").finish_non_exhaustive()
    }
}

impl RenderView for DynRenderView {
    fn render(&mut self, render_pass_context: &mut RenderViewContext) {
        let inner = self.inner.get_mut();
        inner.render(render_pass_context);
    }
}
