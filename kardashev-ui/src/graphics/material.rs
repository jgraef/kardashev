use std::{
    future::Future,
    sync::Arc,
};

use arrayvec::ArrayVec;
use kardashev_protocol::{
    asset_id,
    assets::{
        self as dist,
        AssetId,
    },
};
use palette::WithAlpha;

use super::texture::Texture;
use crate::{
    assets::{
        load::{
            LoadAssetContext,
            LoadFromAsset,
        },
        AssetNotFound,
        MaybeHasAssetId,
    },
    graphics::{
        backend::{
            Backend,
            PerBackend,
        },
        texture::{
            GpuTexture,
            TextureError,
        },
        utils::GpuResourceCache,
    },
    utils::thread_local_cell::ThreadLocalCell,
};

#[derive(Clone, Debug)]
pub struct Material<C> {
    pub asset_id: Option<AssetId>,
    pub label: Option<String>,
    pub cpu: C,
    pub gpu: PerBackend<Arc<ThreadLocalCell<GpuMaterial>>>,
}

impl<C: CpuMaterial> Material<C> {
    pub fn gpu(
        &mut self,
        backend: &Backend,
        cache: &mut GpuResourceCache,
        material_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<&Arc<ThreadLocalCell<GpuMaterial>>, MaterialError> {
        let mut load = |cache: &mut GpuResourceCache| {
            let gpu = self.cpu.load_to_gpu(
                self.label.as_deref(),
                backend,
                material_bind_group_layout,
                cache,
            )?;
            Ok::<_, MaterialError>(Arc::new(ThreadLocalCell::new(gpu)))
        };

        self.gpu.get_or_try_insert(backend.id, || {
            if let Some(asset_id) = self.asset_id {
                if let Some(material) = cache.get(backend.id, asset_id) {
                    Ok(material)
                }
                else {
                    let material = load(cache)?;
                    cache.insert(backend.id, asset_id, &material);
                    Ok(material)
                }
            }
            else {
                load(cache)
            }
        })
    }
}

impl<C> MaybeHasAssetId for Material<C> {
    fn maybe_asset_id(&self) -> Option<AssetId> {
        self.asset_id
    }
}

impl<C: CpuMaterial> LoadFromAsset for Material<C> {
    type Dist = dist::Material;
    type Error = MaterialError;
    type Args = ();

    async fn load<'a, 'b: 'a>(
        asset_id: AssetId,
        _args: (),
        context: &'a mut LoadAssetContext<'b>,
    ) -> Result<Self, Self::Error> {
        let cpu = C::load_from_server(asset_id, context).await?;
        Ok(Material {
            asset_id: Some(asset_id),
            label: None, // todo: move label into the CpuMaterial?
            cpu,
            gpu: PerBackend::default(),
        })
    }
}

pub trait CpuMaterial: Send + Sync + Sized + 'static {
    fn load_from_server<'a, 'b: 'a>(
        asset_id: AssetId,
        context: &'a mut LoadAssetContext<'b>,
    ) -> impl Future<Output = Result<Self, MaterialError>>;

    fn load_to_gpu(
        &mut self,
        label: Option<&str>,
        backend: &Backend,
        material_bind_group_layout: &wgpu::BindGroupLayout,
        cache: &mut GpuResourceCache,
    ) -> Result<GpuMaterial, MaterialError>;
}

#[derive(Debug)]
pub struct GpuMaterial {
    pub bind_group: wgpu::BindGroup,
}

impl GpuMaterial {
    pub fn id(&self) -> GpuMaterialId {
        GpuMaterialId {
            id: self.bind_group.global_id(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuMaterialId {
    id: wgpu::Id<wgpu::BindGroup>,
}

#[derive(Debug, Default)]
pub struct MaterialSamplers {
    pub ambient: Option<Arc<wgpu::Sampler>>,
    pub diffuse: Option<Arc<wgpu::Sampler>>,
    pub specular: Option<Arc<wgpu::Sampler>>,
    pub normal: Option<Arc<wgpu::Sampler>>,
    pub shininess: Option<Arc<wgpu::Sampler>>,
    pub dissolve: Option<Arc<wgpu::Sampler>>,
}

pub struct BindGroupBuilder<'a, 'b, const N: usize> {
    entries: ArrayVec<wgpu::BindGroupEntry<'a>, N>,
    backend: &'b Backend,
    cache: &'b mut GpuResourceCache,
}

impl<'a, 'b: 'a, const N: usize> BindGroupBuilder<'a, 'b, N> {
    pub fn new(backend: &'b Backend, cache: &'b mut GpuResourceCache) -> Self {
        Self {
            entries: ArrayVec::new(),
            backend,
            cache,
        }
    }

    pub fn push(
        &mut self,
        texture: &'a mut Option<Texture>,
        fallback_texture: &'a wgpu::TextureView,
        //sampler: Option<&'a wgpu::Sampler>,
        fallback_sampler: &'a wgpu::Sampler,
    ) -> Result<(), MaterialError> {
        let index = self.entries.len() as u32;

        let texture = texture
            .as_mut()
            .map(|texture| texture.gpu(self.backend, self.cache))
            .transpose()?
            .map(|texture| &texture.get().view)
            .unwrap_or_else(|| fallback_texture);

        //let sampler = sampler.unwrap_or(fallback_sampler);
        let sampler = fallback_sampler;

        self.entries.push(wgpu::BindGroupEntry {
            binding: index,
            resource: wgpu::BindingResource::TextureView(texture),
        });
        self.entries.push(wgpu::BindGroupEntry {
            binding: index + 1,
            resource: wgpu::BindingResource::Sampler(sampler),
        });

        Ok(())
    }

    pub fn build(&self) -> &[wgpu::BindGroupEntry] {
        &self.entries[..]
    }
}

#[derive(Debug, thiserror::Error)]
#[error("load material error")]
pub enum MaterialError {
    AssetNotFound(#[from] AssetNotFound),
    Texture(#[from] TextureError),
    NoCpuMaterial,
}

#[derive(Debug)]
pub struct Fallback {
    pub ambient_texture: Arc<GpuTexture>,
    pub ambient_sampler: Arc<wgpu::Sampler>,
    pub diffuse_texture: Arc<GpuTexture>,
    pub diffuse_sampler: Arc<wgpu::Sampler>,
    pub specular_texture: Arc<GpuTexture>,
    pub specular_sampler: Arc<wgpu::Sampler>,
    pub normal_texture: Arc<GpuTexture>,
    pub normal_sampler: Arc<wgpu::Sampler>,
    pub shininess_texture: Arc<GpuTexture>,
    pub shininess_sampler: Arc<wgpu::Sampler>,
    pub dissolve_texture: Arc<GpuTexture>,
    pub dissolve_sampler: Arc<wgpu::Sampler>,
}

pub fn get_fallback(
    backend: &Backend,
    cache: &mut GpuResourceCache,
) -> Arc<ThreadLocalCell<Fallback>> {
    cache.get_or_insert(
        backend.id,
        asset_id!("916d2b03-eff1-4bc1-a5be-bc3152c9fa75"),
        || {
            let black1x1 = Arc::new(GpuTexture::color1x1(
                palette::named::BLACK.into_format().with_alpha(1.0),
                backend,
            ));
            let pink1x1 = Arc::new(GpuTexture::color1x1(
                palette::named::PINK.into_format().with_alpha(1.0),
                backend,
            ));
            let sampler = Arc::new(backend.device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("fallback sampler"),
                ..Default::default()
            }));
            Arc::new(ThreadLocalCell::new(Fallback {
                ambient_texture: pink1x1.clone(),
                ambient_sampler: sampler.clone(),
                diffuse_texture: pink1x1.clone(),
                diffuse_sampler: sampler.clone(),
                specular_texture: pink1x1.clone(),
                specular_sampler: sampler.clone(),
                normal_texture: black1x1.clone(),
                normal_sampler: sampler.clone(),
                shininess_texture: black1x1.clone(),
                shininess_sampler: sampler.clone(),
                dissolve_texture: black1x1,
                dissolve_sampler: sampler,
            }))
        },
    )
}
