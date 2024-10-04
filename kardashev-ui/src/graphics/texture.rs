use std::sync::Arc;

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
    image_load::{
        self,
        AssetClientLoadImageExt,
    },
    Asset,
    AssetNotFound,
    Loader,
};

#[derive(Clone, Debug)]
pub struct Texture {
    asset_id: Option<AssetId>,
    texture_data: Arc<TextureData>,
}

impl From<RgbaImage> for Texture {
    fn from(image: RgbaImage) -> Self {
        Self {
            asset_id: None,
            texture_data: Arc::new(TextureData { image }),
        }
    }
}

impl Asset for Texture {
    type Dist = dist::Texture;
    type LoadError = LoadTextureError;

    async fn load<'a, 'b: 'a>(
        asset_id: AssetId,
        loader: &'a mut Loader<'b>,
    ) -> Result<Self, Self::LoadError> {
        tracing::debug!(%asset_id, "loading texture");

        let metadata = loader
            .dist_assets
            .get::<dist::Texture>(asset_id)
            .ok_or_else(|| AssetNotFound { asset_id })?;

        if metadata.crop.is_some() {
            todo!("refactor texture atlas system");
        }

        let texture_data = loader
            .cache
            .get_or_try_insert_async(asset_id, || {
                async {
                    let image = loader.client.load_image(&metadata.image).await?;
                    Ok::<_, LoadTextureError>(Arc::new(TextureData { image }))
                }
            })
            .await?;

        tracing::debug!(%asset_id, "texture_loaded");

        Ok(Self {
            asset_id: Some(asset_id),
            texture_data,
        })
    }
}

impl GpuAsset for Texture {
    type Loaded = LoadedTexture;

    fn load(&self, context: &LoadContext) -> Result<Self::Loaded, super::Error> {
        let image = &self.texture_data.as_ref().image;

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
