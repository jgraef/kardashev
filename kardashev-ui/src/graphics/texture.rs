use std::sync::Arc;

use gloo_file::Blob;
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
use crate::{
    assets::{
        image::{
            load_image,
            LoadImageError,
        },
        load::{
            LoadAssetContext,
            LoadFromAsset,
        },
        store::AssetStoreMetaData,
        AssetNotFound,
        MaybeHasAssetId,
    },
    utils::web_fs::{
        self,
        OpenOptions,
    },
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

impl MaybeHasAssetId for Texture {
    fn maybe_asset_id(&self) -> Option<AssetId> {
        self.asset_id
    }
}

impl LoadFromAsset for Texture {
    type Dist = dist::Texture;
    type Error = LoadTextureError;
    type Args = ();

    async fn load<'a, 'b: 'a>(
        asset_id: AssetId,
        _args: (),
        context: &'a mut LoadAssetContext<'b>,
    ) -> Result<Self, Self::Error> {
        tracing::debug!(%asset_id, "loading texture");

        let dist = context
            .dist_assets
            .get::<dist::Texture>(asset_id)
            .ok_or_else(|| AssetNotFound { asset_id })?;

        if dist.crop.is_some() {
            todo!("refactor texture atlas system");
        }

        let texture_data = context
            .cache
            .get_or_try_insert_async(asset_id, || {
                async {
                    let mut file = context
                        .asset_store
                        .open(&dist.image, &OpenOptions::new().create(true))
                        .await?;

                    let mut data = None;

                    if !file.was_created() {
                        let meta_data = file
                            .meta_data()
                            .get::<AssetStoreMetaData>("asset")?
                            .unwrap_or_default();
                        if meta_data.build_time.map_or(false, |t| t >= dist.build_time) {
                            data = Some(file.read_blob().await?);
                        }
                    }

                    let data = if let Some(data) = data {
                        data
                    }
                    else {
                        let fetched_data = context
                            .client
                            .download_file(&dist.image)
                            .await?
                            .bytes()
                            .await?;
                        file.meta_data_mut().insert(
                            "asset",
                            &AssetStoreMetaData {
                                asset_id: Some(dist.id),
                                build_time: Some(dist.build_time),
                            },
                        )?;
                        let fetched_data = Blob::new(fetched_data.as_ref());
                        file.write_blob(fetched_data.clone()).await?;
                        fetched_data
                    };

                    //let image = context.client.load_image(&metadata.image).await?;
                    let image = load_image(data).await?;
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
    LoadImage(#[from] LoadImageError),
    Download(#[from] kardashev_client::DownloadError),
    WebFs(#[from] web_fs::Error),
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
