use bytemuck::{
    Pod,
    Zeroable,
};
use kardashev_protocol::assets::{
    self as dist,
    AssetId,
    Vertex,
};
use palette::Srgb;

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
            Srgb32Ext,
        },
    },
};

#[include_wgsl_oil::include_wgsl_oil("blinn_phong.wgsl")]
mod shader {}

#[derive(Clone, Copy, Debug, Default)]
pub struct CreateBlinnPhongRenderPipeline;

impl CreateRender3dPipeline for CreateBlinnPhongRenderPipeline {
    type Pipeline = BlinnPhongRenderPipeline;

    fn create_pipeline(self, context: &CreateRender3dPipelineContext) -> Self::Pipeline {
        let shader = context
            .backend
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("blinn_phong.wgsl"),
                source: wgpu::ShaderSource::Wgsl(shader::SOURCE.into()),
            });

        let mut material_bind_group_layout_builder = MaterialBindGroupLayoutBuilder::default();
        for _ in 0..7 {
            material_bind_group_layout_builder.push_view_and_sampler();
        }

        let material_bind_group_layout = material_bind_group_layout_builder.build(
            &context.backend.device,
            Some("blinn-phong material bind group"),
        );

        let pipeline_layout =
            context
                .backend
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("blinn-phong pipeline layout"),
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
                    label: Some("blinn-phong pipeline"),
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

        BlinnPhongRenderPipeline {
            pipeline,
            material_bind_group_layout,
            draw_batcher: DrawBatcher::new(context.backend),
        }
    }
}

#[derive(Debug)]
pub struct BlinnPhongRenderPipeline {
    pipeline: wgpu::RenderPipeline,
    material_bind_group_layout: wgpu::BindGroupLayout,
    draw_batcher: DrawBatcher<MeshMaterialPairKey, MeshMaterialPair<BlinnPhongMaterial>, Instance>,
}

impl Render3dPipeline for BlinnPhongRenderPipeline {
    fn render(&mut self, pipeline_context: &mut Render3dPipelineContext) {
        pipeline_context.render_pass.set_pipeline(&self.pipeline);
        pipeline_context.bind_camera_uniform(1);
        pipeline_context.bind_light_uniform(2);
        pipeline_context.batch_meshes_with_material::<BlinnPhongMaterial, Instance>(
            &mut self.draw_batcher,
            &self.material_bind_group_layout,
            |transform, material| {
                Instance {
                    model_transform: transform.as_homogeneous_matrix_array(),
                    material: MaterialInstanceData::from_material(material),
                }
            },
        );
        pipeline_context.draw_batched_meshes_with_materials(&mut self.draw_batcher, 1, 0, 0);
    }
}

#[derive(Clone, Debug, Default)]
pub struct BlinnPhongMaterial {
    pub ambient_texture: Option<Texture>,
    pub ambient_color: Option<Srgb<f32>>,
    pub diffuse_texture: Option<Texture>,
    pub diffuse_color: Option<Srgb<f32>>,
    pub specular_texture: Option<Texture>,
    pub specular_color: Option<Srgb<f32>>,
    pub normal_texture: Option<Texture>,
    pub shininess_texture: Option<Texture>,
    pub shininess: Option<f32>,
    pub dissolve_texture: Option<Texture>,
    pub dissolve: Option<f32>,
    pub emissive_texture: Option<Texture>,
    pub emissive_color: Option<Srgb<f32>>,
}

impl PipelineMaterial for BlinnPhongMaterial {
    async fn load_from_server<'a, 'b: 'a>(
        asset_id: AssetId,
        mut context: &'a mut LoadAssetContext<'b>,
    ) -> Result<Self, MaterialError> {
        tracing::debug!(%asset_id, "loading material");

        // we don't use the cache for materials, since the textures are cached anyway

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

        let ambient_texture = load_material_texture(dist.ambient_texture, &mut context).await?;
        let diffuse_texture = load_material_texture(dist.diffuse_texture, &mut context).await?;
        let specular_texture = load_material_texture(dist.specular_texture, &mut context).await?;
        let normal_texture = load_material_texture(dist.normal_texture, &mut context).await?;
        let shininess_texture = load_material_texture(dist.shininess_texture, &mut context).await?;
        let dissolve_texture = load_material_texture(dist.dissolve_texture, &mut context).await?;
        let emissive_texture = load_material_texture(dist.emissive_texture, &mut context).await?;

        tracing::debug!(%asset_id, "material loaded");

        Ok(Self {
            ambient_texture,
            ambient_color: dist.ambient_color,
            diffuse_texture,
            diffuse_color: dist.diffuse_color,
            specular_texture,
            specular_color: dist.specular_color,
            normal_texture,
            shininess_texture,
            shininess: dist.shininess,
            dissolve_texture,
            dissolve: dist.dissolve,
            emissive_texture,
            emissive_color: dist.emissive_color,
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

        let mut bind_group_builder = BindGroupBuilder::<14>::new(backend, cache);
        bind_group_builder.push(
            &mut self.ambient_texture,
            if self.ambient_color.is_some() {
                &fallback.white.view
            }
            else {
                &fallback.black.view
            },
            &fallback.sampler,
        )?;
        bind_group_builder.push(
            &mut self.diffuse_texture,
            if self.diffuse_color.is_some() {
                &fallback.white.view
            }
            else {
                &fallback.black.view
            },
            &fallback.sampler,
        )?;
        bind_group_builder.push(
            &mut self.specular_texture,
            if self.specular_color.is_some() {
                &fallback.white.view
            }
            else {
                &fallback.black.view
            },
            &fallback.sampler,
        )?;
        bind_group_builder.push(
            &mut self.normal_texture,
            &fallback.normal.view,
            &fallback.sampler,
        )?;
        bind_group_builder.push(
            &mut self.shininess_texture,
            &fallback.white.view,
            &fallback.sampler,
        )?;
        bind_group_builder.push(
            &mut self.dissolve_texture,
            if self.dissolve.is_some() {
                &fallback.white.view
            }
            else {
                &fallback.black.view
            },
            &fallback.sampler,
        )?;
        bind_group_builder.push(
            &mut self.emissive_texture,
            if self.emissive_color.is_some() {
                &fallback.white.view
            }
            else {
                &fallback.black.view
            },
            &fallback.sampler,
        )?;

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
pub struct MaterialInstanceData {
    pub ambient_color: [f32; 3],
    pub diffuse_color: [f32; 3],
    pub specular_color: [f32; 3],
    pub emissive_color: [f32; 3],
    pub shininess: f32,
    pub dissolve: f32,
}

impl MaterialInstanceData {
    pub fn from_material(material: &BlinnPhongMaterial) -> Self {
        const WHITE: Srgb<f32> = Srgb::new(1.0, 1.0, 1.0);
        Self {
            ambient_color: material.ambient_color.unwrap_or(WHITE).as_array3(),
            diffuse_color: material.diffuse_color.unwrap_or(WHITE).as_array3(),
            specular_color: material.specular_color.unwrap_or(WHITE).as_array3(),
            emissive_color: material.emissive_color.unwrap_or(WHITE).as_array3(),
            shininess: material.shininess.unwrap_or(64.0),
            dissolve: material.dissolve.unwrap_or(0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct Instance {
    pub model_transform: [f32; 16],
    pub material: MaterialInstanceData,
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
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // material ambient color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // material diffuse color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // material specular color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // material emissive color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 25]>() as wgpu::BufferAddress,
                    shader_location: 12,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // material shininess
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 28]>() as wgpu::BufferAddress,
                    shader_location: 13,
                    format: wgpu::VertexFormat::Float32,
                },
                // material shininess
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 29]>() as wgpu::BufferAddress,
                    shader_location: 14,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}
