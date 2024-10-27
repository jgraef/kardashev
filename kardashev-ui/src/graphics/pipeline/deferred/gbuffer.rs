use arrayvec::ArrayVec;
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
struct Buffers {
    position: TextureBuffer,
    normal: TextureBuffer,
    diffuse_occlusion: TextureBuffer,
    specular_smoothness: TextureBuffer,
    emission: TextureBuffer,
}

impl Buffers {
    fn as_array(&self) -> [&TextureBuffer; GeometryBuffer::NUM_TEXTURES] {
        [
            &self.position,
            &self.normal,
            &self.diffuse_occlusion,
            &self.specular_smoothness,
            &self.emission,
        ]
    }
}

#[derive(Debug)]
pub struct GeometryBuffer {
    pub size: SurfaceSize,
    buffers: Buffers,
    sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl GeometryBuffer {
    pub const NUM_TEXTURES: usize = 5;

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
        let diffuse_occlusion = TextureBuffer::new(
            backend,
            size,
            wgpu::TextureFormat::Rgba32Float,
            Some("diffuse/occlusion buffer"),
        );
        let specular_smoothness = TextureBuffer::new(
            backend,
            size,
            wgpu::TextureFormat::Rgba32Float,
            Some("specular/smoothness buffer"),
        );
        let emission = TextureBuffer::new(
            backend,
            size,
            wgpu::TextureFormat::Rgba32Float,
            Some("emission buffer"),
        );

        let mut bind_group_layout_entries =
            ArrayVec::<wgpu::BindGroupLayoutEntry, { Self::NUM_TEXTURES + 1 }>::new();
        bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
            count: None,
        });
        for i in 0..Self::NUM_TEXTURES {
            bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: i as u32 + 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                },
                count: None,
            });
        }

        let bind_group_layout =
            backend
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("gbuffer bind group layout"),
                    entries: &bind_group_layout_entries,
                });

        let buffers = Buffers {
            position,
            normal,
            diffuse_occlusion,
            specular_smoothness,
            emission,
        };

        let bind_group = create_bind_group(backend, &bind_group_layout, &sampler, &buffers);

        Self {
            size,
            buffers,
            sampler,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn resize(&mut self, backend: &Backend, size: SurfaceSize) {
        if self.size != size {
            self.buffers.position.resize(backend, size);
            self.buffers.normal.resize(backend, size);
            self.buffers.diffuse_occlusion.resize(backend, size);
            self.buffers.specular_smoothness.resize(backend, size);
            self.buffers.emission.resize(backend, size);
            self.bind_group = create_bind_group(
                backend,
                &self.bind_group_layout,
                &self.sampler,
                &self.buffers,
            );
            self.size = size;
        }
    }

    pub fn fragment_targets(&self) -> [wgpu::ColorTargetState; Self::NUM_TEXTURES] {
        self.buffers.as_array().map(|buffer| {
            wgpu::ColorTargetState {
                format: buffer.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            }
        })
    }

    pub fn color_attachments(&self) -> [ColorAttachment; Self::NUM_TEXTURES] {
        const CLEAR_COLOR: Option<Srgba<f32>> = Some(Srgba::new(0.0, 0.0, 0.0, 0.0));
        self.buffers.as_array().map(|buffer| {
            ColorAttachment {
                texture: &buffer.texture_view,
                clear_color: CLEAR_COLOR,
            }
        })
    }
}

fn create_bind_group(
    backend: &Backend,
    bind_group_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    buffers: &Buffers,
) -> wgpu::BindGroup {
    let mut entries = ArrayVec::<wgpu::BindGroupEntry, { GeometryBuffer::NUM_TEXTURES + 1 }>::new();
    entries.push(wgpu::BindGroupEntry {
        binding: 0,
        resource: wgpu::BindingResource::Sampler(sampler),
    });
    for (i, buffer) in buffers.as_array().iter().enumerate() {
        entries.push(wgpu::BindGroupEntry {
            binding: i as u32 + 1,
            resource: wgpu::BindingResource::TextureView(&buffer.texture_view),
        })
    }

    backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gbuffer bind group"),
            layout: bind_group_layout,
            entries: &entries,
        })
}
