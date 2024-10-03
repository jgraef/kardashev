use std::{
    collections::HashMap,
    sync::Arc,
};

use image::RgbaImage;
use kardashev_protocol::assets::{
    self as dist,
    AssetId,
};
use palette::Srgba;
use wgpu::util::DeviceExt;

use super::{
    loading::{
        GpuAsset,
        LoadContext,
    },
    Backend,
};
use crate::assets::{
    image_load,
    Asset,
    AssetNotFound,
    Loader,
};

#[derive(Clone, Debug)]
pub struct Texture {
    asset_id: Option<AssetId>,
    texture_data: Option<Arc<TextureData>>,
}

impl From<RgbaImage> for Texture {
    fn from(image: RgbaImage) -> Self {
        Self {
            asset_id: None,
            texture_data: Some(Arc::new(TextureData { image })),
        }
    }
}

impl Asset for Texture {
    type Dist = dist::Texture;
    type LoadError = LoadTextureError;

    fn parse_dist_manifest(manifest: &dist::Manifest, refs: &mut HashMap<AssetId, usize>) {
        for (index, texture) in manifest.textures.iter().enumerate() {
            refs.insert(texture.id, index);
        }
    }

    fn get_from_dist_manifest(manifest: &dist::Manifest, index: usize) -> Option<&Self::Dist> {
        manifest.textures.get(index)
    }

    async fn load<'a, 'b: 'a>(
        asset_id: AssetId,
        loader: &'a mut Loader<'b>,
    ) -> Result<Self, Self::LoadError> {
        let metadata = loader.metadata.get::<Texture>(asset_id)?;

        let texture_data = loader
            .cache
            .get_or_try_insert_async(asset_id, || {
                async {
                    let image = image_load::load_image(&metadata.image).await?;
                    Ok::<_, LoadTextureError>(Arc::new(TextureData { image }))
                }
            })
            .await?;

        Ok(Self {
            asset_id: Some(asset_id),
            texture_data: Some(texture_data),
        })
    }
}

impl GpuAsset for Texture {
    type Loaded = LoadedTexture;

    fn load(&self, context: &LoadContext) -> Result<Self::Loaded, super::Error> {
        let image = &self.texture_data.as_ref().unwrap().image;

        let image_size = image.dimensions();
        let texture_size = wgpu::Extent3d {
            width: image_size.0,
            height: image_size.1,
            depth_or_array_layers: 1,
        };

        let texture = context.backend.device.create_texture_with_data(
            &context.backend.queue,
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

        Ok(LoadedTexture {
            texture: Arc::new(texture),
            view: Arc::new(view),
            sampler: context.pipeline.default_sampler.clone(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct TextureData {
    image: RgbaImage,
    // todo: view and sampler info from dist
}

#[derive(Debug, thiserror::Error)]
#[error("load texture error")]
pub enum LoadTextureError {
    AssetNotFound(#[from] AssetNotFound),
    LoadImage(#[from] image_load::LoadImageError),
}

#[derive(Clone, Debug)]
pub struct LoadedTexture {
    pub texture: Arc<wgpu::Texture>,
    pub view: Arc<wgpu::TextureView>,
    pub sampler: Arc<wgpu::Sampler>,
}

impl LoadedTexture {
    pub fn color1x1<C: palette::stimulus::IntoStimulus<u8>>(
        color: Srgba<C>,
        sampler: Arc<wgpu::Sampler>,
        backend: &Backend,
    ) -> Self {
        let color: Srgba<u8> = color.into_format();
        let data = palette::cast::into_array_ref(&color);

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
            data,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        LoadedTexture {
            texture: Arc::new(texture),
            view: Arc::new(view),
            sampler,
        }
    }
}
