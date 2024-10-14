use std::sync::Arc;

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
pub struct Material {
    pub asset_id: Option<AssetId>,
    pub label: Option<String>,
    pub textures: MaterialTextures,
    pub gpu: PerBackend<Arc<ThreadLocalCell<GpuMaterial>>>,
}

impl Material {
    pub fn from_texture(texture: impl Into<Texture>) -> Self {
        let texture = texture.into();

        Self {
            asset_id: None,
            label: None,
            textures: MaterialTextures {
                ambient: Some(texture.clone()),
                diffuse: Some(texture),
                specular: None,
                normal: None,
                shininess: None,
                dissolve: None,
            },
            gpu: PerBackend::default(),
        }
    }

    pub fn gpu(
        &mut self,
        backend: &Backend,
        cache: &mut GpuResourceCache,
        material_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<&Arc<ThreadLocalCell<GpuMaterial>>, MaterialError> {
        let mut load = |cache: &mut GpuResourceCache| {
            Ok::<_, MaterialError>(Arc::new(ThreadLocalCell::new(load_material_to_gpu(
                &mut self.textures,
                &mut Default::default(), // todo
                self.label.as_deref(),
                backend,
                material_bind_group_layout,
                cache,
            )?)))
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

impl MaybeHasAssetId for Material {
    fn maybe_asset_id(&self) -> Option<AssetId> {
        self.asset_id
    }
}

impl LoadFromAsset for Material {
    type Dist = dist::Material;
    type Error = MaterialError;
    type Args = ();

    async fn load<'a, 'b: 'a>(
        asset_id: AssetId,
        _args: (),
        context: &'a mut LoadAssetContext<'b>,
    ) -> Result<Self, Self::Error> {
        load_material_from_server(asset_id, context).await
    }
}

#[derive(Clone, Debug, Default)]
pub struct MaterialTextures {
    pub ambient: Option<Texture>,
    pub diffuse: Option<Texture>,
    pub specular: Option<Texture>,
    pub normal: Option<Texture>,
    pub shininess: Option<Texture>,
    pub dissolve: Option<Texture>,
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

async fn load_material_from_server<'a, 'b: 'a>(
    asset_id: AssetId,
    mut context: &'a mut LoadAssetContext<'b>,
) -> Result<Material, MaterialError> {
    tracing::debug!(%asset_id, "loading material");

    // we don't use the cache for materials, since the textures are cached anyway

    let dist = context
        .dist_assets
        .get::<dist::Material>(asset_id)
        .ok_or_else(|| AssetNotFound { asset_id })?;

    async fn load_material_texture<'a, 'b: 'a>(
        asset_id: Option<AssetId>,
        loader: &'a mut LoadAssetContext<'b>,
    ) -> Result<Option<Texture>, TextureError> {
        if let Some(asset_id) = asset_id {
            Ok(Some(
                <Texture as LoadFromAsset>::load(asset_id, (), loader).await?,
            ))
        }
        else {
            Ok(None)
        }
    }

    let ambient = load_material_texture(dist.ambient, &mut context).await?;
    let diffuse = load_material_texture(dist.diffuse, &mut context).await?;
    let specular = load_material_texture(dist.specular, &mut context).await?;
    let normal = load_material_texture(dist.normal, &mut context).await?;
    let shininess = load_material_texture(dist.shininess, &mut context).await?;
    let dissolve = load_material_texture(dist.dissolve, &mut context).await?;

    tracing::debug!(%asset_id, "material loaded");

    Ok(Material {
        asset_id: Some(asset_id),
        label: dist.label.clone(),
        textures: MaterialTextures {
            ambient,
            diffuse,
            specular,
            normal,
            shininess,
            dissolve,
        },
        gpu: PerBackend::default(),
    })
}

fn load_material_to_gpu(
    textures: &mut MaterialTextures,
    samplers: &mut MaterialSamplers,
    label: Option<&str>,
    backend: &Backend,
    material_bind_group_layout: &wgpu::BindGroupLayout,
    cache: &mut GpuResourceCache,
) -> Result<GpuMaterial, MaterialError> {
    struct BindGroupBuilder<'a, 'b> {
        entries: ArrayVec<wgpu::BindGroupEntry<'a>, 12>,
        backend: &'b Backend,
        cache: &'b mut GpuResourceCache,
    }

    impl<'a, 'b: 'a> BindGroupBuilder<'a, 'b> {
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
            sampler: Option<&'a wgpu::Sampler>,
            fallback_sampler: &'a wgpu::Sampler,
        ) -> Result<(), MaterialError> {
            let index = self.entries.len() as u32;

            let texture = texture
                .as_mut()
                .map(|texture| texture.gpu(self.backend, self.cache))
                .transpose()?
                .map(|texture| &texture.get().view)
                .unwrap_or_else(|| fallback_texture);

            let sampler = sampler.unwrap_or(fallback_sampler);

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

    let fallback = get_fallback(backend, cache);
    let fallback = fallback.get();

    let mut bind_group_builder = BindGroupBuilder::new(backend, cache);
    bind_group_builder.push(
        &mut textures.ambient,
        &fallback.ambient_texture.view,
        samplers.ambient.as_deref(),
        &fallback.ambient_sampler,
    )?;
    bind_group_builder.push(
        &mut textures.diffuse,
        &fallback.diffuse_texture.view,
        samplers.diffuse.as_deref(),
        &fallback.diffuse_sampler,
    )?;
    bind_group_builder.push(
        &mut textures.specular,
        &fallback.specular_texture.view,
        samplers.specular.as_deref(),
        &fallback.specular_sampler,
    )?;
    bind_group_builder.push(
        &mut textures.normal,
        &fallback.normal_texture.view,
        samplers.normal.as_deref(),
        &fallback.normal_sampler,
    )?;
    bind_group_builder.push(
        &mut textures.shininess,
        &fallback.shininess_texture.view,
        samplers.shininess.as_deref(),
        &fallback.shininess_sampler,
    )?;
    bind_group_builder.push(
        &mut textures.dissolve,
        &fallback.dissolve_texture.view,
        samplers.dissolve.as_deref(),
        &fallback.dissolve_sampler,
    )?;

    let bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            layout: material_bind_group_layout,
            entries: bind_group_builder.build(),
            label: label.clone(),
        });

    Ok(GpuMaterial { bind_group })
}

#[derive(Debug, thiserror::Error)]
#[error("load material error")]
pub enum MaterialError {
    AssetNotFound(#[from] AssetNotFound),
    Texture(#[from] TextureError),
    NoCpuMaterial,
}

#[derive(Debug)]
struct Fallback {
    ambient_texture: Arc<GpuTexture>,
    ambient_sampler: Arc<wgpu::Sampler>,
    diffuse_texture: Arc<GpuTexture>,
    diffuse_sampler: Arc<wgpu::Sampler>,
    specular_texture: Arc<GpuTexture>,
    specular_sampler: Arc<wgpu::Sampler>,
    normal_texture: Arc<GpuTexture>,
    normal_sampler: Arc<wgpu::Sampler>,
    shininess_texture: Arc<GpuTexture>,
    shininess_sampler: Arc<wgpu::Sampler>,
    dissolve_texture: Arc<GpuTexture>,
    dissolve_sampler: Arc<wgpu::Sampler>,
}

fn get_fallback(backend: &Backend, cache: &mut GpuResourceCache) -> Arc<ThreadLocalCell<Fallback>> {
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
