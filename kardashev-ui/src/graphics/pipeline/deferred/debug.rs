use bytemuck::{
    Pod,
    Zeroable,
};

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

#[derive(Clone, Copy, Debug)]
pub enum Channel {
    Position,
    Normal,
    Diffuse,
    Occlusion,
    Specular,
    Shininess,
    Emission,
}

impl Channel {
    fn as_u32(&self) -> u32 {
        match self {
            Channel::Position => 0,
            Channel::Normal => 1,
            Channel::Diffuse => 2,
            Channel::Occlusion => 3,
            Channel::Specular => 4,
            Channel::Shininess => 5,
            Channel::Emission => 6,
        }
    }
}

#[derive(Debug)]
pub struct DebugPipeline {
    pub channel: Channel,
    debug_uniform: UniformBuffer<DebugUniform>,
    pipeline: wgpu::RenderPipeline,
}

impl DebugPipeline {
    pub fn new(
        context: &CreatePipelineContext,
        globals: &UniformBuffer<GlobalsUniform>,
        geometry_buffer: &GeometryBuffer,
        output_config: &TextureConfig,
    ) -> Self {
        let debug_uniform = UniformBuffer::new(context.backend);

        let pipeline = PipelineBuilder::new(shader::SOURCE)
            .with_label("deferred lighting pipeline")
            .with_bind_group_layout(&globals.bind_group_layout)
            .with_bind_group_layout(&geometry_buffer.bind_group_layout)
            .with_bind_group_layout(&debug_uniform.bind_group_layout)
            .with_fragment_target(wgpu::ColorTargetState {
                format: output_config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })
            .build(context.backend);

        Self {
            channel: Channel::Position,
            debug_uniform,
            pipeline,
        }
    }
}

impl RenderPipeline for DebugPipeline {
    type Input<'a> = (&'a GeometryBuffer, &'a UniformBuffer<GlobalsUniform>);
    type Output<'a> = TextureOutput<'a>;

    fn render(
        &mut self,
        context: &mut RenderPipelineContext,
        (geometry_buffer, globals): Self::Input<'_>,
        output: Self::Output<'_>,
    ) {
        self.debug_uniform.write(
            context.backend,
            &DebugUniform {
                channel: self.channel.as_u32(),
                _padding: Default::default(),
            },
        );
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
        render_pass.set_bind_group(2, &self.debug_uniform.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
#[repr(C)]
struct DebugUniform {
    channel: u32,
    _padding: [u32; 3],
}

#[include_wgsl_oil::include_wgsl_oil("debug.wgsl")]
mod shader {}
