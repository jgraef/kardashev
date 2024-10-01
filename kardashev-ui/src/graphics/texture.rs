use std::sync::Arc;

use image::RgbaImage;
use palette::Srgba;
use wgpu::util::DeviceExt;

use super::Backend;

#[derive(Clone, Debug)]
pub struct Texture {
    pub texture: Arc<wgpu::Texture>,
    pub view: Arc<wgpu::TextureView>,
    pub sampler: Arc<wgpu::Sampler>,
}

impl Texture {
    pub fn load(image: &RgbaImage, sampler: Arc<wgpu::Sampler>, backend: &Backend) -> Self {
        let image_size = image.dimensions();
        let texture_size = wgpu::Extent3d {
            width: image_size.0,
            height: image_size.1,
            depth_or_array_layers: 1,
        };

        let texture = backend.device.create_texture_with_data(
            &backend.queue,
            &wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: None,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::default(),
            image.as_raw(),
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Texture {
            texture: Arc::new(texture),
            view: Arc::new(view),
            sampler,
        }
    }

    pub fn color1x1<C: palette::stimulus::IntoStimulus<u8>>(
        color: Srgba<C>,
        sampler: Arc<wgpu::Sampler>,
        backend: &Backend,
    ) -> Self {
        let color: Srgba<u8> = color.into_format();
        let data = [color.red, color.green, color.blue, color.alpha];

        let texture = backend.device.create_texture_with_data(
            &backend.queue,
            &wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("black 1x1"),
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::default(),
            &data,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Texture {
            texture: Arc::new(texture),
            view: Arc::new(view),
            sampler,
        }
    }
}
