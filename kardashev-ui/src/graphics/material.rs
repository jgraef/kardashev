use std::sync::Arc;

use kardashev_protocol::assets::{
    self as dist,
    AssetId,
};

use super::{
    loading::{
        GpuAsset,
        LoadContext,
    },
    texture::Texture,
};
use crate::{
    assets::{
        load::{
            LoadAssetContext,
            LoadFromAsset,
        },
        AssetNotFound,
        MaybeHasAssetId,
    },
    graphics::texture::{
        LoadTextureError,
        LoadedTexture,
    },
};

#[derive(Clone, Debug)]
pub struct Material {
    pub asset_id: Option<AssetId>,
    pub material_data: Arc<MaterialData>,
}

impl Material {
    pub fn from_diffuse(diffuse: impl Into<Texture>) -> Self {
        Self {
            asset_id: None,
            material_data: Arc::new(MaterialData {
                diffuse: Some(diffuse.into()),
                ..Default::default()
            }),
        }
    }
}

impl MaybeHasAssetId for Material {
    fn maybe_asset_id(&self) -> Option<AssetId> {
        self.asset_id
    }
}

impl LoadFromAsset for Material {
    type Dist = dist::Material;
    type Error = LoadMaterialError;
    type Args = ();

    async fn load<'a, 'b: 'a>(
        asset_id: AssetId,
        _args: (),
        mut context: &'a mut LoadAssetContext<'b>,
    ) -> Result<Self, Self::Error> {
        tracing::debug!(%asset_id, "loading material");

        // we don't use the cache for materials, since the textures are cached anyway

        let metadata = context
            .dist_assets
            .get::<dist::Material>(asset_id)
            .ok_or_else(|| AssetNotFound { asset_id })?;

        async fn load_material_texture<'a, 'b: 'a>(
            asset_id: Option<AssetId>,
            loader: &'a mut LoadAssetContext<'b>,
        ) -> Result<Option<Texture>, LoadTextureError> {
            if let Some(asset_id) = asset_id {
                Ok(Some(
                    <Texture as LoadFromAsset>::load(asset_id, (), loader).await?,
                ))
            }
            else {
                Ok(None)
            }
        }

        let ambient = load_material_texture(metadata.ambient, &mut context).await?;
        let diffuse = load_material_texture(metadata.diffuse, &mut context).await?;
        let specular = load_material_texture(metadata.specular, &mut context).await?;
        let normal = load_material_texture(metadata.normal, &mut context).await?;
        let shininess = load_material_texture(metadata.shininess, &mut context).await?;
        let dissolve = load_material_texture(metadata.dissolve, &mut context).await?;

        let material_data = MaterialData {
            ambient,
            diffuse,
            specular,
            normal,
            shininess,
            dissolve,
        };

        tracing::debug!(%asset_id, "material loaded");

        Ok(Self {
            asset_id: Some(asset_id),
            material_data: Arc::new(material_data),
        })
    }
}

impl GpuAsset for Material {
    type Loaded = LoadedMaterial;

    fn load(&self, context: &LoadContext) -> Result<Self::Loaded, super::Error> {
        let load_texture =
            |texture: &Option<Texture>| -> Result<Option<LoadedTexture>, super::Error> {
                texture
                    .as_ref()
                    .map(|texture| <Texture as GpuAsset>::load(texture, context))
                    .transpose()
            };

        let ambient = load_texture(&self.material_data.ambient)?;
        let diffuse = load_texture(&self.material_data.diffuse)?;
        let specular = load_texture(&self.material_data.specular)?;
        let normal = load_texture(&self.material_data.normal)?;
        let shininess = load_texture(&self.material_data.shininess)?;
        let dissolve = load_texture(&self.material_data.dissolve)?;

        fn texture_view_bind_group_entry<'a>(
            binding: u32,
            texture: &'a Option<LoadedTexture>,
            fallback: &'a LoadedTexture,
        ) -> wgpu::BindGroupEntry<'a> {
            wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::TextureView(
                    texture
                        .as_ref()
                        .map(|texture| &texture.view)
                        .unwrap_or(&fallback.view),
                ),
            }
        }

        fn texture_sampler_bind_group_entry<'a>(
            binding: u32,
            texture: &'a Option<LoadedTexture>,
            fallback: &'a LoadedTexture,
        ) -> wgpu::BindGroupEntry<'a> {
            wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::Sampler(
                    texture
                        .as_ref()
                        .map(|texture| &texture.sampler)
                        .unwrap_or(&fallback.sampler),
                ),
            }
        }

        let bind_group = context
            .backend
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &context.pipeline.material_bind_group_layout,
                entries: &[
                    texture_view_bind_group_entry(0, &ambient, &context.pipeline.fallback_texture),
                    texture_sampler_bind_group_entry(
                        1,
                        &ambient,
                        &context.pipeline.fallback_texture,
                    ),
                    texture_view_bind_group_entry(2, &diffuse, &context.pipeline.fallback_texture),
                    texture_sampler_bind_group_entry(
                        3,
                        &diffuse,
                        &context.pipeline.fallback_texture,
                    ),
                    texture_view_bind_group_entry(4, &specular, &context.pipeline.fallback_texture),
                    texture_sampler_bind_group_entry(
                        5,
                        &specular,
                        &context.pipeline.fallback_texture,
                    ),
                    texture_view_bind_group_entry(6, &normal, &context.pipeline.fallback_texture),
                    texture_sampler_bind_group_entry(
                        7,
                        &normal,
                        &context.pipeline.fallback_texture,
                    ),
                    texture_view_bind_group_entry(
                        8,
                        &shininess,
                        &context.pipeline.fallback_texture,
                    ),
                    texture_sampler_bind_group_entry(
                        9,
                        &shininess,
                        &context.pipeline.fallback_texture,
                    ),
                    texture_view_bind_group_entry(
                        10,
                        &dissolve,
                        &context.pipeline.fallback_texture,
                    ),
                    texture_sampler_bind_group_entry(
                        11,
                        &dissolve,
                        &context.pipeline.fallback_texture,
                    ),
                ],
                label: Some("material bind group"),
            });

        Ok(LoadedMaterial { bind_group })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("load material error")]
pub enum LoadMaterialError {
    AssetNotFound(#[from] AssetNotFound),
    LoadTexture(#[from] LoadTextureError),
}

#[derive(Debug)]
pub struct LoadedMaterial {
    pub bind_group: wgpu::BindGroup,
}

#[derive(Clone, Debug, Default)]
pub struct MaterialData {
    pub ambient: Option<Texture>,
    pub diffuse: Option<Texture>,
    pub specular: Option<Texture>,
    pub normal: Option<Texture>,
    pub shininess: Option<Texture>,
    pub dissolve: Option<Texture>,
}
