pub mod debug;
pub mod gbuffer;
pub mod geometry;
pub mod lighting;

use crate::graphics::{
    pipeline::{
        deferred::{
            debug::{
                Channel,
                DebugPipeline,
            },
            gbuffer::GeometryBuffer,
            geometry::GeometryPipeline,
            lighting::LightingPipeline,
        },
        draw_world::DrawWorldGlobals,
        globals::GlobalsUniform,
        CreatePipeline,
        RenderPipeline,
        RenderPipelineContext,
        RenderWorldInput,
        TextureConfig,
        TextureOutput,
    },
    utils::UniformBuffer,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct CreateDeferredPipeline;

impl CreatePipeline for CreateDeferredPipeline {
    type Pipeline = DeferredPipeline;
    type InputConfig = ();
    type OutputConfig = TextureConfig;

    fn create_pipeline(
        self,
        context: &super::CreatePipelineContext,
        _input_config: &mut Self::InputConfig,
        output_config: &mut Self::OutputConfig,
    ) -> Self::Pipeline {
        let globals = UniformBuffer::new(context.backend);
        let geometry_buffer = GeometryBuffer::new(context.backend, output_config.size);
        let geometry_pipeline = GeometryPipeline::new(context, &globals, &geometry_buffer);
        let lighting_pipeline =
            LightingPipeline::new(context, &globals, &geometry_buffer, &output_config);
        let debug_pipeline =
            DebugPipeline::new(context, &globals, &geometry_buffer, &output_config);

        DeferredPipeline {
            geometry_buffer,
            globals,
            geometry_pipeline,
            lighting_pipeline,
            debug_pipeline,
            debug: false,
        }
    }
}

#[derive(Debug)]
pub struct DeferredPipeline {
    geometry_buffer: GeometryBuffer,
    globals: UniformBuffer<GlobalsUniform>,
    geometry_pipeline: GeometryPipeline,
    lighting_pipeline: LightingPipeline,
    debug_pipeline: DebugPipeline,
    debug: bool,
}

impl DeferredPipeline {
    pub fn set_debug(&mut self, channel: Option<Channel>) {
        if let Some(channel) = channel {
            self.debug = true;
            self.debug_pipeline.channel = channel;
        }
        else {
            self.debug = false;
        }
    }
}

impl RenderPipeline for DeferredPipeline {
    type Input<'a> = RenderWorldInput<'a>;
    type Output<'a> = TextureOutput<'a>;

    fn render(
        &mut self,
        context: &mut RenderPipelineContext,
        input: Self::Input<'_>,
        output: Self::Output<'_>,
    ) {
        self.geometry_buffer.resize(context.backend, output.size);

        if let Some(globals) = DrawWorldGlobals::from_world(&input) {
            self.globals.write(context.backend, &globals.globals);

            self.geometry_pipeline
                .render(context, (input, &self.globals), &self.geometry_buffer);
            if self.debug {
                self.debug_pipeline
                    .render(context, (&self.geometry_buffer, &self.globals), output);
            }
            else {
                self.lighting_pipeline.render(
                    context,
                    (&self.geometry_buffer, &self.globals),
                    output,
                );
            }
        }
    }
}
