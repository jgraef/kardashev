use bytemuck::{
    Pod,
    Zeroable,
};
use kardashev_protocol::assets::{
    self as dist,
    AssetId,
    Vertex,
};

use crate::{
    assets::{
        load::{
            LoadAssetContext,
            LoadFromAsset,
        },
        AssetNotFound,
    },
    graphics::{
        draw_batch::DrawBatcher,
        material::{
            get_fallback,
            BindGroupBuilder,
            GpuMaterial,
            MaterialError,
            PipelineMaterial,
        },
        render_3d::{
            CreateRender3dPipeline,
            CreateRender3dPipelineContext,
            MeshMaterialPair,
            MeshMaterialPairKey,
            Render3dPipeline,
            Render3dPipelineContext,
        },
        texture::{
            Texture,
            TextureError,
        },
        utils::{
            GpuResourceCache,
            HasVertexBufferLayout,
            MaterialBindGroupLayoutBuilder,
        },
    },
};

#[include_wgsl_oil::include_wgsl_oil("pbr.wgsl")]
mod shader {}

#[derive(Clone, Copy, Debug, Default)]
pub struct CreatePbrRenderPipeline;

impl CreateRender3dPipeline for CreatePbrRenderPipeline {
    type Pipeline = PbrRenderPipeline;

    fn create_pipeline(self, context: &CreateRender3dPipelineContext) -> Self::Pipeline {
        let shader = context
            .backend
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("pbr.wgsl"),
                source: wgpu::ShaderSource::Wgsl(shader::SOURCE.into()),
            });

        let mut material_bind_group_layout_builder = MaterialBindGroupLayoutBuilder::default();
        for _ in 0..4 {
            material_bind_group_layout_builder.push_view_and_sampler();
        }

        let material_bind_group_layout = material_bind_group_layout_builder
            .build(&context.backend.device, Some("pbr material bind group"));

        let pipeline_layout =
            context
                .backend
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("pbr pipeline layout"),
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
                    label: Some("pbr pipeline"),
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
    draw_batcher: DrawBatcher<MeshMaterialPairKey, MeshMaterialPair<PbrMaterial>, Instance>,
}

impl Render3dPipeline for PbrRenderPipeline {
    fn render(&mut self, pipeline_context: &mut Render3dPipelineContext) {
        pipeline_context.render_pass.set_pipeline(&self.pipeline);
        pipeline_context.bind_camera_uniform(1);
        pipeline_context.bind_light_uniform(2);
        pipeline_context.batch_meshes_with_material::<PbrMaterial, Instance>(
            &mut self.draw_batcher,
            &self.material_bind_group_layout,
            |transform, _| {
                Instance {
                    model_transform: transform.as_homogeneous_matrix_array(),
                }
            },
        );
        pipeline_context.draw_batched_meshes_with_materials(&mut self.draw_batcher, 1, 0, 0);
    }
}

#[derive(Clone, Debug, Default)]
pub struct PbrMaterial {
    pub albedo: Option<Texture>,
    pub normal: Option<Texture>,
    pub metalness: Option<Texture>,
    pub roughness: Option<Texture>,
}

impl PipelineMaterial for PbrMaterial {
    async fn load_from_server<'a, 'b: 'a>(
        asset_id: AssetId,
        mut context: &'a mut LoadAssetContext<'b>,
    ) -> Result<Self, MaterialError> {
        tracing::debug!(%asset_id, "loading material");

        let dist = context
            .dist_assets
            .get::<dist::Material>(asset_id)
            .ok_or_else(|| AssetNotFound { asset_id })?;

        async fn load_material_texture<'a, 'b: 'a>(
            asset_id: Option<AssetId>,
            loader: &'a mut LoadAssetContext<'b>,
        ) -> Result<Option<Texture>, TextureError> {
            if let Some(asset_id) = asset_id {
                Ok(Some(
                    <Texture as LoadFromAsset>::load(asset_id, (), loader).await?,
                ))
            }
            else {
                Ok(None)
            }
        }

        let albedo = load_material_texture(dist.albedo_texture, &mut context).await?;
        let normal = load_material_texture(dist.normal_texture, &mut context).await?;
        let metalness = load_material_texture(dist.metalness_texture, &mut context).await?;
        let roughness = load_material_texture(dist.roughness_texture, &mut context).await?;

        tracing::debug!(%asset_id, "material loaded");

        Ok(Self {
            albedo,
            normal,
            metalness,
            roughness,
        })
    }

    fn load_to_gpu(
        &mut self,
        label: Option<&str>,
        backend: &super::backend::Backend,
        material_bind_group_layout: &wgpu::BindGroupLayout,
        cache: &mut GpuResourceCache,
    ) -> Result<GpuMaterial<Self>, MaterialError> {
        let fallback = get_fallback(backend, cache);
        let fallback = fallback.get();

        let mut bind_group_builder = BindGroupBuilder::<8>::new(backend, cache);
        bind_group_builder.push(&mut self.albedo, &fallback.pink.view, &fallback.sampler)?;
        bind_group_builder.push(&mut self.normal, &fallback.black.view, &fallback.sampler)?;
        bind_group_builder.push(&mut self.metalness, &fallback.black.view, &fallback.sampler)?;
        bind_group_builder.push(&mut self.roughness, &fallback.black.view, &fallback.sampler)?;

        let bind_group = backend
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: material_bind_group_layout,
                entries: bind_group_builder.build(),
                label: label.clone(),
            });

        Ok(GpuMaterial::new(bind_group))
    }
}

#[derive(Clone, Copy, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct Instance {
    pub model_transform: [f32; 16],
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
            ],
        }
    }
}
