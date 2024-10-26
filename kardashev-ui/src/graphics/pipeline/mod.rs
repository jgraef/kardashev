//pub mod deferred;
pub mod draw_world;
pub mod forward;
pub mod globals;
pub mod hdr;

use crate::{
    ecs::resource::Resources,
    graphics::{
        backend::Backend,
        SurfaceSize,
    },
};

pub trait CreatePipeline {
    type Pipeline: RenderPipeline;
    type InputConfig;
    type OutputConfig;

    fn create_pipeline(
        self,
        context: &CreatePipelineContext,
        input_config: &mut Self::InputConfig,
        output_config: &mut Self::OutputConfig,
    ) -> Self::Pipeline;
}

pub trait RenderPipeline {
    type Input<'a>;
    type Output<'a>;

    fn render(
        &mut self,
        context: &mut RenderPipelineContext,
        input: Self::Input<'_>,
        output: Self::Output<'_>,
    );
}

#[derive(Debug)]
pub struct CreatePipelineContext<'a> {
    pub backend: &'a Backend,
}

#[derive(Debug)]
pub struct RenderPipelineContext<'a> {
    pub backend: &'a Backend,
    pub encoder: &'a mut wgpu::CommandEncoder,
}

#[derive(Clone, Copy, Debug)]
pub struct TextureConfig {
    pub format: wgpu::TextureFormat,
    pub size: SurfaceSize,
    pub usages: wgpu::TextureUsages,
}

impl TextureConfig {
    pub fn new(format: wgpu::TextureFormat, size: SurfaceSize) -> Self {
        Self {
            format,
            size,
            usages: wgpu::TextureUsages::empty(),
        }
    }

    pub fn add_usages(&mut self, usages: wgpu::TextureUsages) -> &mut Self {
        self.usages.set(usages, true);
        self
    }

    pub fn with_usages(mut self, usages: wgpu::TextureUsages) -> Self {
        self.add_usages(usages);
        self
    }
}

#[derive(Debug)]
pub struct TextureOutput<'a> {
    pub view: &'a wgpu::TextureView,
    pub size: SurfaceSize,
}

// todo: Debug
pub struct RenderWorldInput<'a> {
    pub view_entity: hecs::Entity,
    pub world: &'a hecs::World,
    pub resources: &'a mut Resources,
}
