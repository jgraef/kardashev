use bytemuck::{
    Pod,
    Zeroable,
};
use palette::Srgba;

use crate::graphics::{
    render_3d::{
        CreateRender3dPipeline,
        CreateRender3dPipelineContext,
        Render3dPipeline,
        Render3dPipelineContext,
    },
    transform::GlobalTransform,
    utils::{
        color_to_array,
        HasVertexBufferLayout,
        InstanceBuffer,
    },
};

#[derive(Debug)]
pub struct Star {
    pub color: Srgba<f32>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CreateRenderStarPipeline;

impl CreateRender3dPipeline for CreateRenderStarPipeline {
    type Pipeline = RenderStarPipeline;

    fn create_pipeline(self, context: &CreateRender3dPipelineContext) -> Self::Pipeline {
        let shader = context
            .backend
            .device
            .create_shader_module(wgpu::include_wgsl!("./shader.wgsl"));

        let pipeline_layout =
            context
                .backend
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("RenderStarPipeline pipeline layout"),
                    bind_group_layouts: &[&context.camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let pipeline =
            context
                .backend
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("RenderStarPipeline pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[Instance::layout()],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: context.surface_format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: context.depth_texture_format,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        RenderStarPipeline {
            pipeline,
            instance_buffer: InstanceBuffer::new(context.backend, 128),
        }
    }
}

#[derive(Debug)]
pub struct RenderStarPipeline {
    pipeline: wgpu::RenderPipeline,
    instance_buffer: InstanceBuffer<Instance>,
}

impl Render3dPipeline for RenderStarPipeline {
    fn render(&mut self, context: &mut Render3dPipelineContext) {
        let mut query = context.world.query::<(&GlobalTransform, &Star)>();

        for (_entity, (transform, star)) in query.iter() {
            self.instance_buffer.push(Instance {
                model_transform: transform.as_homogeneous_matrix_array(),
                color: color_to_array(star.color),
            });
        }

        let num_instances = self.instance_buffer.len().try_into().unwrap();
        if num_instances > 0 {
            tracing::trace!(num_instances, "drawing stars");

            self.instance_buffer.upload_and_clear(&context.backend);

            context.render_pass.set_pipeline(&self.pipeline);
            context
                .render_pass
                .set_bind_group(0, &context.camera_bind_group, &[]);
            context
                .render_pass
                .set_vertex_buffer(0, self.instance_buffer.slice(..));
            context.render_pass.draw(0..6, 0..num_instances);
        }
    }
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct Instance {
    model_transform: [f32; 16],
    color: [f32; 4],
}

impl HasVertexBufferLayout for Instance {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
