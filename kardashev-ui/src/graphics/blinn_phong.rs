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
        texture::{
            Texture,
            TextureError,
        },
        transform::GlobalTransform,
        utils::{
            GpuResourceCache,
            HasVertexBufferLayout,
            MaterialBindGroupLayoutBuilder,
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
    draw_batcher: DrawBatcher<Instance>,
}

impl Render3dPipeline for BlinnPhongRenderPipeline {
    fn render(&mut self, pipeline_context: &mut Render3dPipelineContext) {
        pipeline_context.render_pass.set_pipeline(&self.pipeline);

        pipeline_context
            .render_pass
            .set_bind_group(1, &pipeline_context.camera_bind_group, &[]);
        pipeline_context
            .render_pass
            .set_bind_group(2, &pipeline_context.light_bind_group, &[]);

        tracing::trace!("batching");

        let mut render_entities = pipeline_context.world.query::<(
            &GlobalTransform,
            &mut Mesh,
            &mut Material<BlinnPhongMaterial>,
        )>();

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

#[derive(Clone, Debug, Default)]
pub struct BlinnPhongMaterial {
    pub ambient: Option<Texture>,
    pub diffuse: Option<Texture>,
    pub specular: Option<Texture>,
    pub normal: Option<Texture>,
    pub shininess: Option<Texture>,
    pub dissolve: Option<Texture>,
}

impl CpuMaterial for BlinnPhongMaterial {
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

        let ambient = load_material_texture(dist.ambient, &mut context).await?;
        let diffuse = load_material_texture(dist.diffuse, &mut context).await?;
        let specular = load_material_texture(dist.specular, &mut context).await?;
        let normal = load_material_texture(dist.normal, &mut context).await?;
        let shininess = load_material_texture(dist.shininess, &mut context).await?;
        let dissolve = load_material_texture(dist.dissolve, &mut context).await?;

        tracing::debug!(%asset_id, "material loaded");

        Ok(Self {
            ambient,
            diffuse,
            specular,
            normal,
            shininess,
            dissolve,
        })
    }

    fn load_to_gpu(
        &mut self,
        label: Option<&str>,
        backend: &super::backend::Backend,
        material_bind_group_layout: &wgpu::BindGroupLayout,
        cache: &mut GpuResourceCache,
    ) -> Result<super::material::GpuMaterial, MaterialError> {
        let fallback = get_fallback(backend, cache);
        let fallback = fallback.get();

        let mut bind_group_builder = BindGroupBuilder::<12>::new(backend, cache);
        bind_group_builder.push(
            &mut self.ambient,
            &fallback.ambient_texture.view,
            &fallback.ambient_sampler,
        )?;
        bind_group_builder.push(
            &mut self.diffuse,
            &fallback.diffuse_texture.view,
            &fallback.diffuse_sampler,
        )?;
        bind_group_builder.push(
            &mut self.specular,
            &fallback.specular_texture.view,
            &fallback.specular_sampler,
        )?;
        bind_group_builder.push(
            &mut self.normal,
            &fallback.normal_texture.view,
            &fallback.normal_sampler,
        )?;
        bind_group_builder.push(
            &mut self.shininess,
            &fallback.shininess_texture.view,
            &fallback.shininess_sampler,
        )?;
        bind_group_builder.push(
            &mut self.dissolve,
            &fallback.dissolve_texture.view,
            &fallback.dissolve_sampler,
        )?;

        let bind_group = backend
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: material_bind_group_layout,
                entries: bind_group_builder.build(),
                label: label.clone(),
            });

        Ok(GpuMaterial { bind_group })
    }
}
