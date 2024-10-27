use bytemuck::{
    Pod,
    Zeroable,
};
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
        PipelineBuilder,
        RenderPassBuilder,
        TextureBuffer,
        UniformBuffer,
    },
    SurfaceSize,
};

#[derive(Clone, Copy, Debug, strum::IntoStaticStr, strum::VariantNames)]
pub enum ToneMap {
    Aces,
    Reinard,
    Exposure { exposure: f32 },
}

#[derive(Clone, Copy, Debug)]
pub struct CreateHdrPipeline<C> {
    pub inner: C,
    pub format: wgpu::TextureFormat,
    pub tone_map: ToneMap,
    pub gamma: f32,
}

impl<C> CreateHdrPipeline<C> {
    pub fn new(inner: C) -> Self {
        Self {
            inner,
            format: wgpu::TextureFormat::Rgba16Float,
            tone_map: ToneMap::Aces,
            gamma: 1.0,
        }
    }

    pub fn with_tone_map(mut self, tone_map: ToneMap) -> Self {
        self.tone_map = tone_map;
        self
    }

    pub fn with_gamma(mut self, gamma: f32) -> Self {
        self.gamma = gamma;
        self
    }
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
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(SamplerBindingType::NonFiltering),
                            count: None,
                        },
                    ],
                });

        let config_uniform = UniformBuffer::new(context.backend);

        let pipeline = PipelineBuilder::new(shader::SOURCE)
            .with_label("hdr pipeline")
            .with_bind_group_layout(&config_uniform.bind_group_layout)
            .with_bind_group_layout(&bind_group_layout)
            .with_fragment_target(wgpu::ColorTargetState {
                format: output_config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })
            .build(context.backend);

        let staging_texture =
            StagingTexture::new(context.backend, &inner_output_config, bind_group_layout);

        HdrPipeline {
            inner,
            staging_texture,
            pipeline,
            config_uniform,
            tone_map: self.tone_map,
            gamma: self.gamma,
        }
    }
}

impl<P> HdrPipeline<P> {
    pub fn set_tone_map(&mut self, tone_map: ToneMap) {
        self.tone_map = tone_map;
    }

    pub fn set_gamma(&mut self, gamma: f32) {
        self.gamma = gamma;
    }
}

#[derive(Debug)]
pub struct HdrPipeline<P> {
    inner: P,
    staging_texture: StagingTexture,
    pipeline: wgpu::RenderPipeline,
    config_uniform: UniformBuffer<ConfigUniform>,
    tone_map: ToneMap,
    gamma: f32,
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

        self.config_uniform.write(
            context.backend,
            &ConfigUniform::new(self.tone_map, self.gamma),
        );

        let mut render_pass_builder = RenderPassBuilder::<1>::default();
        render_pass_builder.with_label("hdr tonemapping render pass");
        render_pass_builder.with_color_attachment(ColorAttachment {
            texture: output.view,
            clear_color: None,
        });
        let mut render_pass = render_pass_builder.begin(context.encoder);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.config_uniform.bind_group, &[]);
        render_pass.set_bind_group(1, &self.staging_texture.bind_group, &[]);
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

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct ConfigUniform {
    tone_map: u32,
    exposure: f32,
    gamma: f32,
    _padding: u32,
}

impl ConfigUniform {
    pub fn new(tone_map: ToneMap, gamma: f32) -> Self {
        let mut config = ConfigUniform::zeroed();
        config.gamma = gamma;

        config.tone_map = match tone_map {
            ToneMap::Aces => 0,
            ToneMap::Reinard => 1,
            ToneMap::Exposure { exposure } => {
                config.exposure = exposure;
                2
            }
        };

        config
    }
}

#[include_wgsl_oil::include_wgsl_oil("hdr.wgsl")]
mod shader {}
