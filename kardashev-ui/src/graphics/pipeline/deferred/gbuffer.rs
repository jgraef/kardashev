use palette::Srgba;

use crate::graphics::{
    backend::Backend,
    utils::{
        ColorAttachment,
        TextureBuffer,
    },
    SurfaceSize,
};

#[derive(Debug)]
pub struct GeometryBuffer {
    pub size: SurfaceSize,
    pub position: TextureBuffer,
    pub normal: TextureBuffer,
    pub diffuse_specular: TextureBuffer,
    pub sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl GeometryBuffer {
    pub const NUM_TEXTURES: usize = 3;

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
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
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
            self.size = size;
        }
    }

    pub fn fragment_targets(&self) -> [wgpu::ColorTargetState; Self::NUM_TEXTURES] {
        [
            // position
            wgpu::ColorTargetState {
                format: self.position.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            },
            // normal
            wgpu::ColorTargetState {
                format: self.normal.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            },
            // diffuse/specular
            wgpu::ColorTargetState {
                format: self.diffuse_specular.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            },
        ]
    }

    pub fn color_attachments(&self) -> [ColorAttachment; Self::NUM_TEXTURES] {
        const CLEAR_COLOR: Option<Srgba<f32>> = Some(Srgba::new(0.0, 0.0, 0.0, 0.0));
        [
            ColorAttachment {
                texture: &self.position.texture_view,
                clear_color: CLEAR_COLOR,
            },
            ColorAttachment {
                texture: &self.normal.texture_view,
                clear_color: CLEAR_COLOR,
            },
            ColorAttachment {
                texture: &self.diffuse_specular.texture_view,
                clear_color: CLEAR_COLOR,
            },
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
