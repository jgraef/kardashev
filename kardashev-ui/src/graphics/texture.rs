use std::sync::Arc;

use gloo_file::Blob;
use image::RgbaImage;
use kardashev_client::AssetClient;
use kardashev_protocol::assets::{
    self as dist,
    AssetId,
};
use palette::Srgba;
use wgpu::util::DeviceExt;

use super::Backend;
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
        store::{
            AssetStoreGuard,
            AssetStoreMetaData,
        },
        AssetNotFound,
        MaybeHasAssetId,
    },
    graphics::{
        backend::PerBackend,
        utils::GpuResourceCache,
    },
    utils::{
        thread_local_cell::ThreadLocalCell,
        web_fs::{
            self,
            OpenOptions,
        },
    },
};

#[derive(Clone, Debug)]
pub struct Texture {
    asset_id: Option<AssetId>,
    label: Option<String>,
    cpu: Option<Arc<CpuTexture>>,
    gpu: PerBackend<Arc<ThreadLocalCell<GpuTexture>>>,
}

impl Texture {
    pub fn cpu(&self) -> Option<&CpuTexture> {
        self.cpu.as_deref()
    }

    pub fn gpu(
        &mut self,
        backend: &Backend,
        cache: &mut GpuResourceCache,
    ) -> Result<&Arc<ThreadLocalCell<GpuTexture>>, TextureError> {
        self.gpu.get_or_try_insert(backend.id, || {
            let texture_data = self
                .cpu
                .as_ref()
                .ok_or_else(|| TextureError::NoCpuTexture)?;
            if let Some(asset_id) = self.asset_id {
                cache.get_or_try_insert(backend.id, asset_id, || {
                    Ok::<_, TextureError>(Arc::new(ThreadLocalCell::new(load_texture_to_gpu(
                        texture_data,
                        self.label.as_deref(),
                        backend,
                    )?)))
                })
            }
            else {
                Ok::<_, TextureError>(Arc::new(ThreadLocalCell::new(load_texture_to_gpu(
                    texture_data,
                    self.label.as_deref(),
                    backend,
                )?)))
            }
        })
    }
}

impl From<RgbaImage> for Texture {
    fn from(image: RgbaImage) -> Self {
        Self {
            asset_id: None,
            label: None,
            cpu: Some(Arc::new(CpuTexture { image })),
            gpu: PerBackend::default(),
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
    type Error = TextureError;
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

        let texture = context
            .cache
            .get_or_try_insert_async(asset_id, || {
                load_texture_from_server(dist, &context.asset_store, &context.client)
            })
            .await?;

        tracing::debug!(%asset_id, "texture_loaded");

        Ok(Self {
            asset_id: Some(asset_id),
            label: dist.label.clone(),
            cpu: Some(texture),
            gpu: PerBackend::default(),
        })
    }
}

pub(super) async fn load_texture_from_server(
    dist: &dist::Texture,
    asset_store: &AssetStoreGuard,
    client: &AssetClient,
) -> Result<Arc<CpuTexture>, TextureError> {
    let mut file = asset_store
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
        let fetched_data = client.download_file(&dist.image).await?.bytes().await?;
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
    Ok::<_, TextureError>(Arc::new(CpuTexture { image }))
}

fn load_texture_to_gpu(
    texture: &CpuTexture,
    label: Option<&str>,
    backend: &Backend,
) -> Result<GpuTexture, TextureError> {
    let image = &texture.image;

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
            label,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::default(),
        image.as_raw(),
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    Ok(GpuTexture { texture, view })
}

#[derive(Clone, Debug)]
pub struct CpuTexture {
    image: RgbaImage,
    // todo: view and sampler info from dist
}

#[derive(Debug, thiserror::Error)]
#[error("load texture error")]
pub enum TextureError {
    AssetNotFound(#[from] AssetNotFound),
    LoadImage(#[from] LoadImageError),
    Download(#[from] kardashev_client::DownloadError),
    WebFs(#[from] web_fs::Error),
    NoCpuTexture,
}

#[derive(Debug)]
pub struct GpuTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl GpuTexture {
    pub fn id(&self) -> GpuTextureId {
        GpuTextureId {
            texture: self.texture.global_id(),
            view: self.view.global_id(),
        }
    }

    pub fn color1x1<C: palette::stimulus::IntoStimulus<u8>>(
        color: Srgba<C>,
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

        GpuTexture { texture, view }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuTextureId {
    texture: wgpu::Id<wgpu::Texture>,
    view: wgpu::Id<wgpu::TextureView>,
}
