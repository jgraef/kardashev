use std::sync::Arc;

use image::RgbaImage;
use kardashev_protocol::assets::AssetId;
use linear_map::LinearMap;
use wgpu::util::DeviceExt;

use super::BackendId;

#[derive(Debug)]
pub struct Material {
    asset_id: AssetId,
    //data: Option<Arc<RwLock<MaterialData>>>,
}

struct MaterialData {}

#[derive(Clone, Debug)]
struct LoadedMaterial {
    ambient: Option<LoadedTexture>,
    diffuse: Option<LoadedTexture>,
    specular: Option<LoadedTexture>,
    normal: Option<LoadedTexture>,
    shininess: Option<LoadedTexture>,
    dissolve: Option<LoadedTexture>,
    bind_group: Arc<wgpu::BindGroup>,
}

#[derive(Clone, Debug)]
struct LoadedTexture {
    texture: Arc<wgpu::Texture>,
    view: Arc<wgpu::TextureView>,
    sampler: Arc<wgpu::Sampler>,
}

#[derive(Debug)]
pub struct LoaderContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub material_bind_group_layout: &'a wgpu::BindGroupLayout,
}

impl Material {
    pub fn load(image: &RgbaImage, context: &LoaderContext) -> Self {
        let image_size = image.dimensions();
        let texture_size = wgpu::Extent3d {
            width: image_size.0,
            height: image_size.1,
            depth_or_array_layers: 1,
        };

        let diffuse_texture = context.device.create_texture_with_data(
            &context.queue,
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

        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let diffuse_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let _bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &context.material_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                    },
                ],
                label: Some("diffuse_bind_group"),
            });

        todo!();
    }
}
