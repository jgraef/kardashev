use wgpu::SamplerBindingType;

use crate::graphics::{
    backend::Backend,
    render_frame::{
        CreateRenderPass,
        CreateRenderPassContext,
        RenderPass,
        RenderPassContext,
    },
    SurfaceSize,
};

#[derive(Clone, Copy, Debug)]
pub struct CreateToneMapPass<P: CreateRenderPass> {
    pub inner: P,
    pub format: wgpu::TextureFormat,
}

impl<P: CreateRenderPass> CreateRenderPass for CreateToneMapPass<P> {
    type RenderPass = ToneMapPass<P::RenderPass>;

    fn create_render_pass(self, context: &CreateRenderPassContext) -> Self::RenderPass {
        let inner = self.inner.create_render_pass(&CreateRenderPassContext {
            backend: context.backend,
            surface_size: context.surface_size,
            surface_format: self.format,
        });

        let tone_mapping = ToneMapPipeline::new(context.backend, context.surface_format);
        let staging = StagingTexture::new(
            context.backend,
            context.surface_size,
            self.format,
            &tone_mapping.bind_group_layout,
        );

        ToneMapPass {
            inner,
            staging,
            tone_mapping,
        }
    }
}

#[derive(Debug)]
pub struct ToneMapPass<P> {
    inner: P,
    staging: StagingTexture,
    tone_mapping: ToneMapPipeline,
}

impl<P: RenderPass> RenderPass for ToneMapPass<P> {
    fn render(&mut self, context: &mut RenderPassContext) {
        self.staging.resize_if_needed(
            context.backend,
            context.target_size,
            &self.tone_mapping.bind_group_layout,
        );

        self.inner.render(&mut RenderPassContext {
            backend: context.backend,
            encoder: context.encoder,
            target_view: &self.staging.view,
            target_size: context.target_size,
            render_target_entity: context.render_target_entity,
            world: context.world,
            resources: context.resources,
        });

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("hdr tonemapping render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: context.target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        render_pass.set_pipeline(&self.tone_mapping.pipeline);
        render_pass.set_bind_group(0, &self.staging.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

#[derive(Debug)]
struct StagingTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    format: wgpu::TextureFormat,
    bind_group: wgpu::BindGroup,
}

impl StagingTexture {
    fn new(
        backend: &Backend,
        size: SurfaceSize,
        format: wgpu::TextureFormat,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let (texture, view) = create_staging_texture(backend, size, format);
        let sampler = backend.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("hdr staging sampler"),
            ..Default::default()
        });
        let bind_group = create_staging_bind_group(backend, &view, &sampler, bind_group_layout);

        Self {
            texture,
            view,
            sampler,
            format,
            bind_group,
        }
    }

    fn resize(
        &mut self,
        backend: &Backend,
        size: SurfaceSize,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        let (texture, view) = create_staging_texture(backend, size, self.format);
        self.texture = texture;
        self.view = view;
        self.bind_group =
            create_staging_bind_group(backend, &self.view, &self.sampler, bind_group_layout);
    }

    fn resize_if_needed(
        &mut self,
        backend: &Backend,
        size: SurfaceSize,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        if SurfaceSize::from_texture(&self.texture) != size {
            tracing::debug!(?size, "resizing staging texture");
            self.resize(backend, size, bind_group_layout);
        }
    }
}

fn create_staging_texture(
    backend: &Backend,
    size: SurfaceSize,
    format: wgpu::TextureFormat,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = backend.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("hdr staging texture"),
        size: wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let view = texture.create_view(&Default::default());

    (texture, view)
}

fn create_staging_bind_group(
    backend: &Backend,
    view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hdr staging bind group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        })
}

#[derive(Debug)]
struct ToneMapPipeline {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
}

impl ToneMapPipeline {
    fn new(backend: &Backend, format: wgpu::TextureFormat) -> Self {
        let shader = backend
            .device
            .create_shader_module(wgpu::include_wgsl!("hdr.wgsl"));

        let bind_group_layout =
            backend
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("hdr staging bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let pipeline_layout =
            backend
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("hdr tonemapping pipeline layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        let pipeline = backend
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("hdr tonemapping pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    compilation_options: Default::default(),
                    buffers: &[],
                },
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
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                multiview: None,
                cache: None,
            });

        Self {
            bind_group_layout,
            pipeline_layout,
            pipeline,
        }
    }
}
