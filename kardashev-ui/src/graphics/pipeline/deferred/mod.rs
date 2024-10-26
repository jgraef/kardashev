use kardashev_protocol::assets::Vertex;

use crate::graphics::{
    backend::Backend,
    camera::{
        CameraProjection,
        ClearColor,
    },
    draw_batch::DrawBatcher,
    light::PointLight,
    pipeline::{
        draw_world::{
            MeshMaterialPair,
            MeshMaterialPairKey,
        },
        forward::blinn_phong::{
            BlinnPhongMaterial,
            Instance,
            MaterialInstanceData,
        },
        globals::GlobalsUniform,
    },
    transform::GlobalTransform,
    utils::{
        HasVertexBufferLayout,
        MaterialBindGroupLayoutBuilder,
        Srgba64Ext,
        TextureBuffer,
        UniformBuffer,
    },
    SurfaceSize,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct CreateDeferredRenderPass;

impl CreateRenderPass for CreateDeferredRenderPass {
    type RenderPass = DeferredRenderPass;

    fn create_render_pass(self, context: &CreateRenderPassContext) -> Self::RenderPass {
        let depth_texture = TextureBuffer::new(
            context.backend,
            context.surface_size,
            wgpu::TextureFormat::Depth32Float,
            Some("depth texture"),
        );

        let g_buffer = GBuffer::new(context.backend, context.surface_size);

        let globals = UniformBuffer::new(context.backend);

        let geometry_shader =
            context
                .backend
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("geometry.wgsl"),
                    source: wgpu::ShaderSource::Wgsl(geometry_shader::SOURCE.into()),
                });

        let lighting_shader =
            context
                .backend
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("lighting.wgsl"),
                    source: wgpu::ShaderSource::Wgsl(lighting_shader::SOURCE.into()),
                });

        let mut material_bind_group_layout_builder = MaterialBindGroupLayoutBuilder::default();
        for _ in 0..7 {
            material_bind_group_layout_builder.push_view_and_sampler();
        }
        let material_bind_group_layout = material_bind_group_layout_builder
            .build(&context.backend.device, Some("material bind group"));

        let geometry_pipeline_layout =
            context
                .backend
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("geomtry pipeline layout"),
                    bind_group_layouts: &[&globals.bind_group_layout, &material_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let geometry_pipeline =
            context
                .backend
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("geometry pipeline"),
                    layout: Some(&geometry_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &geometry_shader,
                        entry_point: "vs_main",
                        buffers: &[Vertex::layout(), Instance::layout()],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &geometry_shader,
                        entry_point: "fs_main",
                        targets: &g_buffer.layout_targets(),
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
                        format: depth_texture.format,
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

        let mut gbuffers_bind_group_layout_builder = MaterialBindGroupLayoutBuilder::default();
        for _ in 0..4 {
            gbuffers_bind_group_layout_builder.push_view_and_sampler();
        }
        let gbuffers_bind_group_layout = gbuffers_bind_group_layout_builder
            .build(&context.backend.device, Some("gbuffers bind group"));

        let lighting_pipeline_layout =
            context
                .backend
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("geomtry pipeline layout"),
                    bind_group_layouts: &[&globals.bind_group_layout, &gbuffers_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let lighting_pipeline =
            context
                .backend
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("lighting pipeline"),
                    layout: Some(&lighting_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &lighting_shader,
                        entry_point: "vs_main",
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &lighting_shader,
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
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        DeferredRenderPass {
            depth_texture,
            g_buffer,
            globals,
            geometry_pipeline,
            lighting_pipeline,
            material_bind_group_layout,
            draw_batcher: DrawBatcher::new(&context.backend),
        }
    }
}

#[derive(Debug)]
pub struct DeferredRenderPass {
    depth_texture: TextureBuffer,
    g_buffer: GBuffer,
    globals: UniformBuffer<GlobalsUniform>,
    geometry_pipeline: wgpu::RenderPipeline,
    lighting_pipeline: wgpu::RenderPipeline,
    material_bind_group_layout: wgpu::BindGroupLayout,
    draw_batcher: DrawBatcher<MeshMaterialPairKey, MeshMaterialPair<BlinnPhongMaterial>, Instance>,
}

impl RenderPass for DeferredRenderPass {
    fn render(&mut self, context: &mut RenderViewContext) {
        self.depth_texture
            .resize(context.backend, context.target_size);
        self.g_buffer.resize(context.backend, context.target_size);

        let mut query_camera = context
            .world
            .query_one::<(Option<&ClearColor>, &GlobalTransform, &CameraProjection)>(
                context.render_target_entity,
            )
            .expect("render target entity doesn't exist");

        if let Some((clear_color, camera_transform, camera_projection)) = query_camera.get() {
            let mut globals = GlobalsUniform::default();

            // set camera
            globals.set_camera(camera_projection, camera_transform);

            // query lights
            let mut query_lights = context.world.query::<(&GlobalTransform, &PointLight)>();
            for (_, (transform, point_light)) in query_lights.iter() {
                if !globals.add_point_light(transform.position(), point_light.color) {
                    break;
                }
            }

            self.globals.write(context.backend, &globals);

            // geometry render pass
            {
                let mut geometry_render_pass =
                    context
                        .encoder
                        .begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("geometry render pass"),
                            color_attachments: &self.g_buffer.color_attachments(),
                            depth_stencil_attachment: Some(
                                wgpu::RenderPassDepthStencilAttachment {
                                    view: &self.depth_texture.texture_view,
                                    depth_ops: Some(wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(1.0),
                                        store: wgpu::StoreOp::Store,
                                    }),
                                    stencil_ops: None,
                                },
                            ),
                            occlusion_query_set: None,
                            timestamp_writes: None,
                        });

                geometry_render_pass.set_pipeline(&self.geometry_pipeline);
                geometry_render_pass.set_bind_group(0, &self.globals.bind_group, &[]);

                batch_meshes_with_material(
                    context.world,
                    context.resources,
                    context.backend,
                    &mut self.draw_batcher,
                    &self.material_bind_group_layout,
                    |transform, material| {
                        Instance {
                            model_transform: transform.as_homogeneous_matrix_array(),
                            material: MaterialInstanceData::from_material(material),
                        }
                    },
                );

                draw_batched_meshes_with_materials(
                    &mut geometry_render_pass,
                    context.backend,
                    &mut self.draw_batcher,
                    1,
                    0,
                    1,
                );
            }

            // lighting render pass
            {
                let mut lighting_render_pass =
                    context
                        .encoder
                        .begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("lighting render pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: context.target_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: clear_color
                                        .map(|c| {
                                            wgpu::LoadOp::Clear(
                                                c.clear_color.into_format().as_wgpu(),
                                            )
                                        })
                                        .unwrap_or(wgpu::LoadOp::Load),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            occlusion_query_set: None,
                            timestamp_writes: None,
                        });

                lighting_render_pass.set_pipeline(&self.lighting_pipeline);
                lighting_render_pass.set_bind_group(0, &self.globals.bind_group, &[]);
                lighting_render_pass.set_bind_group(1, &self.g_buffer.bind_group, &[]);
                lighting_render_pass.draw(0..3, 0..1);
            }
        }
        else {
            tracing::warn!("entity with RenderTarget component is missing other camera components");
        }
    }
}

#[derive(Debug)]
struct GBuffer {
    pub size: SurfaceSize,
    pub position: TextureBuffer,
    pub normal: TextureBuffer,
    pub diffuse_specular: TextureBuffer,
    pub sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl GBuffer {
    pub fn new(backend: &Backend, size: SurfaceSize) -> Self {
        let sampler = backend.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("gbuffer sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        let position = TextureBuffer::new(
            backend,
            size,
            wgpu::TextureFormat::Rgba32Float,
            Some("position buffer"),
        );
        let normal = TextureBuffer::new(
            backend,
            size,
            wgpu::TextureFormat::Rgba32Float,
            Some("normal buffer"),
        );
        let diffuse_specular = TextureBuffer::new(
            backend,
            size,
            wgpu::TextureFormat::Rgba32Float,
            Some("diffuse/specular buffer"),
        );

        let bind_group_layout =
            backend
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("gbuffer bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                    ],
                });

        let bind_group = Self::create_bind_group(
            backend,
            &bind_group_layout,
            &sampler,
            &position,
            &normal,
            &diffuse_specular,
        );

        Self {
            size,
            position,
            normal,
            diffuse_specular,
            sampler,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn resize(&mut self, backend: &Backend, size: SurfaceSize) {
        if self.size != size {
            self.position.resize(backend, size);
            self.normal.resize(backend, size);
            self.diffuse_specular.resize(backend, size);
            self.bind_group = Self::create_bind_group(
                backend,
                &self.bind_group_layout,
                &self.sampler,
                &self.position,
                &self.normal,
                &self.diffuse_specular,
            );
        }
    }

    pub fn layout_targets(&self) -> [Option<wgpu::ColorTargetState>; 3] {
        [
            // position
            Some(wgpu::ColorTargetState {
                format: self.position.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }),
            // normal
            Some(wgpu::ColorTargetState {
                format: self.normal.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }),
            // diffuse/specular
            Some(wgpu::ColorTargetState {
                format: self.diffuse_specular.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }),
        ]
    }

    pub fn color_attachments(&self) -> [Option<wgpu::RenderPassColorAttachment>; 3] {
        [
            Some(wgpu::RenderPassColorAttachment {
                view: &self.position.texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &self.normal.texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &self.diffuse_specular.texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            }),
        ]
    }

    fn create_bind_group(
        backend: &Backend,
        bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        position: &TextureBuffer,
        normal: &TextureBuffer,
        diffuse_specular: &TextureBuffer,
    ) -> wgpu::BindGroup {
        backend
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("gbuffer bind group"),
                layout: bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&position.texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&normal.texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(
                            &diffuse_specular.texture_view,
                        ),
                    },
                ],
            })
    }
}

#[include_wgsl_oil::include_wgsl_oil("geometry.wgsl")]
mod geometry_shader {}

#[include_wgsl_oil::include_wgsl_oil("lighting.wgsl")]
mod lighting_shader {}
