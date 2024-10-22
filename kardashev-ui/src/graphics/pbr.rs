use bytemuck::{
    Pod,
    Zeroable,
};
use kardashev_protocol::assets::{
    AssetId,
    Vertex,
};

use crate::{
    assets::load::LoadAssetContext,
    graphics::{
        backend::Backend,
        draw_batch::DrawBatcher,
        material::{
            CpuMaterial,
            GpuMaterial,
            Material,
            MaterialError,
        },
        mesh::Mesh,
        render_3d::{
            CreateRender3dPipeline,
            CreateRender3dPipelineContext,
            Render3dPipeline,
            Render3dPipelineContext,
        },
        transform::GlobalTransform,
        utils::{
            GpuResourceCache,
            HasVertexBufferLayout,
            MaterialBindGroupLayoutBuilder,
        },
    },
};

#[derive(Clone, Copy, Debug, Default)]
pub struct CreatePbrRenderPipeline;

impl CreateRender3dPipeline for CreatePbrRenderPipeline {
    type Pipeline = PbrRenderPipeline;

    fn create_pipeline(self, context: &CreateRender3dPipelineContext) -> Self::Pipeline {
        let shader = context
            .backend
            .device
            .create_shader_module(wgpu::include_wgsl!("pbr.wgsl"));

        let mut material_bind_group_layout_builder = MaterialBindGroupLayoutBuilder::default();
        for _ in 0..6 {
            material_bind_group_layout_builder.push_view_and_sampler();
        }

        let material_bind_group_layout = material_bind_group_layout_builder
            .build(&context.backend.device, Some("material bind group"));

        let pipeline_layout =
            context
                .backend
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render3dMeshesWithMaterial pipeline layout"),
                    bind_group_layouts: &[
                        &material_bind_group_layout,
                        &context.camera_bind_group_layout,
                        &context.light_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let pipeline =
            context
                .backend
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render3dMeshesWithMaterial pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[Vertex::layout(), Instance::layout()],
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

        PbrRenderPipeline {
            pipeline,
            material_bind_group_layout,
            draw_batcher: DrawBatcher::new(context.backend),
        }
    }
}

#[derive(Debug)]
pub struct PbrRenderPipeline {
    pipeline: wgpu::RenderPipeline,
    material_bind_group_layout: wgpu::BindGroupLayout,
    draw_batcher: DrawBatcher<Instance>,
}

impl Render3dPipeline for PbrRenderPipeline {
    fn render(&mut self, pipeline_context: &mut Render3dPipelineContext) {
        pipeline_context.render_pass.set_pipeline(&self.pipeline);

        pipeline_context
            .render_pass
            .set_bind_group(1, &pipeline_context.camera_bind_group, &[]);
        pipeline_context
            .render_pass
            .set_bind_group(2, &pipeline_context.light_bind_group, &[]);

        tracing::trace!("batching");

        let mut render_entities =
            pipeline_context
                .world
                .query::<(&GlobalTransform, &mut Mesh, &mut Material<PbrMaterial>)>();

        let gpu_resource_cache = pipeline_context
            .resources
            .get_mut_or_insert_default::<GpuResourceCache>();

        for (entity, (transform, mesh, material)) in render_entities.iter() {
            tracing::trace!(?entity, ?mesh, ?material, "rendering entity");

            // handle errors

            let Ok(mesh) = mesh.gpu(&pipeline_context.backend, gpu_resource_cache)
            else {
                continue;
            };

            let Ok(material) = material.gpu(
                &pipeline_context.backend,
                gpu_resource_cache,
                &self.material_bind_group_layout,
            )
            else {
                continue;
            };

            self.draw_batcher
                .push(mesh, material, Instance::from_transform(transform));
        }

        self.draw_batcher
            .draw(pipeline_context.backend, pipeline_context.render_pass);
    }
}

#[derive(Clone, Copy, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct Instance {
    pub model_transform: [f32; 16],
    // note: we're using the trick mentioned here[1] to rotate the vertex normal by the rotation of
    // the model matrix: https://sotrh.github.io/learn-wgpu/intermediate/tutorial10-lighting/#the-normal-matrix
    //pub normal: [f32; 9],
}

impl Instance {
    pub fn from_transform(transform: &GlobalTransform) -> Self {
        Self {
            model_transform: transform.as_homogeneous_matrix_array(),
            /*normal: transform
            .model_matrix
            .isometry
            .rotation
            .to_rotation_matrix()
            .matrix()
            .as_slice()
            .try_into()
            .expect("convert rotation matrix to array"),*/
        }
    }
}

impl HasVertexBufferLayout for Instance {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // model transform
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // normal
                /*wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },*/
            ],
        }
    }
}

#[derive(Debug)]
pub struct PbrMaterial {
    // todo
}

impl CpuMaterial for PbrMaterial {
    async fn load_from_server<'a, 'b: 'a>(
        _asset_id: AssetId,
        _context: &'a mut LoadAssetContext<'b>,
    ) -> Result<Self, MaterialError> {
        todo!()
    }

    fn load_to_gpu(
        &mut self,
        _label: Option<&str>,
        _backend: &Backend,
        _material_bind_group_layout: &wgpu::BindGroupLayout,
        _cache: &mut GpuResourceCache,
    ) -> Result<GpuMaterial, MaterialError> {
        todo!()
    }
}
