use crate::graphics::{
    pipeline::{
        deferred::gbuffer::GeometryBuffer,
        globals::GlobalsUniform,
        CreatePipelineContext,
        RenderPipeline,
        RenderPipelineContext,
        TextureConfig,
        TextureOutput,
    },
    utils::{
        ColorAttachment,
        PipelineBuilder,
        RenderPassBuilder,
        UniformBuffer,
    },
};

#[derive(Debug)]
pub struct LightingPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl LightingPipeline {
    pub fn new(
        context: &CreatePipelineContext,
        globals: &UniformBuffer<GlobalsUniform>,
        geometry_buffer: &GeometryBuffer,
        output_config: &TextureConfig,
    ) -> Self {
        let pipeline = PipelineBuilder::new(shader::SOURCE)
            .with_label("deferred lighting pipeline")
            .with_bind_group_layout(&globals.bind_group_layout)
            .with_bind_group_layout(&geometry_buffer.bind_group_layout)
            .with_fragment_target(wgpu::ColorTargetState {
                format: output_config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })
            .build(context.backend);

        Self { pipeline }
    }
}

impl RenderPipeline for LightingPipeline {
    type Input<'a> = (&'a GeometryBuffer, &'a UniformBuffer<GlobalsUniform>);
    type Output<'a> = TextureOutput<'a>;

    fn render(
        &mut self,
        context: &mut RenderPipelineContext,
        (geometry_buffer, globals): Self::Input<'_>,
        output: Self::Output<'_>,
    ) {
        let mut render_pass = RenderPassBuilder::<1>::default()
            .with_label("deferred lighting render pass")
            .with_color_attachment(ColorAttachment {
                texture: output.view,
                clear_color: None,
            })
            .begin(context.encoder);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &globals.bind_group, &[]);
        render_pass.set_bind_group(1, &geometry_buffer.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

#[include_wgsl_oil::include_wgsl_oil("lighting.wgsl")]
mod shader {}
