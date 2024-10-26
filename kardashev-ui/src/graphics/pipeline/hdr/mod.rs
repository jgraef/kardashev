use wgpu::SamplerBindingType;

use crate::graphics::{
    backend::Backend,
    pipeline::{
        CreatePipeline,
        RenderPipeline,
        RenderPipelineContext,
        TextureConfig,
        TextureOutput,
    },
    utils::{
        ColorAttachment,
        RenderPassBuilder,
        TextureBuffer,
    },
    SurfaceSize,
};

#[derive(Clone, Copy, Debug)]
pub enum ToneMap {
    Aces,
    Gamma { gamma: f32 }, // todo
}

#[derive(Clone, Copy, Debug)]
pub struct CreateHdrPipeline<C> {
    pub inner: C,
    pub format: wgpu::TextureFormat,
    pub tone_map: ToneMap,
}

impl<C, P> CreatePipeline for CreateHdrPipeline<C>
where
    C: CreatePipeline<Pipeline = P, OutputConfig = TextureConfig>,
    for<'a> P: RenderPipeline<Output<'a> = TextureOutput<'a>>,
{
    type Pipeline = HdrPipeline<P>;
    type InputConfig = C::InputConfig;
    type OutputConfig = TextureConfig;

    fn create_pipeline(
        self,
        context: &super::CreatePipelineContext,
        input_config: &mut Self::InputConfig,
        output_config: &mut Self::OutputConfig,
    ) -> Self::Pipeline {
        let mut inner_output_config = TextureConfig {
            format: self.format,
            size: output_config.size,
            usages: wgpu::TextureUsages::TEXTURE_BINDING,
        };

        let inner = self
            .inner
            .create_pipeline(context, input_config, &mut inner_output_config);

        let shader = context
            .backend
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("hdr.wgsl"),
                source: wgpu::ShaderSource::Wgsl(shader::SOURCE.into()),
            });

        let bind_group_layout =
            context
                .backend
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
            context
                .backend
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("hdr tonemapping pipeline layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        let pipeline =
            context
                .backend
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
                            format: output_config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    multiview: None,
                    cache: None,
                });

        let staging_texture =
            StagingTexture::new(context.backend, &inner_output_config, bind_group_layout);

        HdrPipeline {
            inner,
            staging_texture,
            pipeline,
        }
    }
}

#[derive(Debug)]
pub struct HdrPipeline<P> {
    inner: P,
    staging_texture: StagingTexture,
    pipeline: wgpu::RenderPipeline,
}

impl<P> RenderPipeline for HdrPipeline<P>
where
    for<'a> P: RenderPipeline<Output<'a> = TextureOutput<'a>>,
{
    type Input<'a> = P::Input<'a>;
    type Output<'a> = TextureOutput<'a>;

    fn render(
        &mut self,
        context: &mut super::RenderPipelineContext,
        input: Self::Input<'_>,
        output: Self::Output<'_>,
    ) {
        self.staging_texture.resize(context.backend, output.size);

        self.inner.render(
            &mut RenderPipelineContext {
                backend: context.backend,
                encoder: context.encoder,
            },
            input,
            TextureOutput {
                view: &self.staging_texture.texture.texture_view,
                size: output.size,
            },
        );

        let mut render_pass_builder = RenderPassBuilder::<1>::default();
        render_pass_builder.with_label("hdr tonemapping render pass");
        render_pass_builder.with_color_attachment(ColorAttachment {
            texture: output.view,
            clear_color: None,
        });
        let mut render_pass = render_pass_builder.begin(context.encoder);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.staging_texture.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

#[derive(Debug)]
struct StagingTexture {
    texture: TextureBuffer,
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl StagingTexture {
    fn new(
        backend: &Backend,
        config: &TextureConfig,
        bind_group_layout: wgpu::BindGroupLayout,
    ) -> Self {
        let texture = TextureBuffer::new(
            backend,
            config.size,
            config.format,
            Some("hdr staging texture"),
        );
        let sampler = backend.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("hdr staging sampler"),
            ..Default::default()
        });
        let bind_group =
            create_staging_bind_group(backend, &texture.texture_view, &sampler, &bind_group_layout);
        Self {
            texture,
            sampler,
            bind_group,
            bind_group_layout,
        }
    }

    fn resize(&mut self, backend: &Backend, size: SurfaceSize) {
        if self.texture.resize(backend, size) {
            self.bind_group = create_staging_bind_group(
                backend,
                &self.texture.texture_view,
                &self.sampler,
                &self.bind_group_layout,
            );
        }
    }
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

#[include_wgsl_oil::include_wgsl_oil("hdr.wgsl")]
mod shader {}
