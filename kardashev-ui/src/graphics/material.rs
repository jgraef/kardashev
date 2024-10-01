use std::{
    hash::Hash,
    sync::Arc,
};

use image::RgbaImage;
use kardashev_protocol::assets::AssetId;
use linear_map::LinearMap;

use super::{
    rendering_system::LoadContext,
    texture::Texture,
    BackendId,
};
use crate::utils::thread_local_cell::ThreadLocalCell;

#[derive(Debug)]
pub struct Material {
    asset_id: Option<AssetId>,
    data: Option<MaterialData>,
    loaded: LinearMap<BackendId, LoadedMaterial>,
}

impl Material {
    pub(super) fn loaded(&mut self, context: &LoadContext) -> Option<&LoadedMaterial> {
        match self.loaded.entry(context.backend.id()) {
            linear_map::Entry::Occupied(occupied) => Some(occupied.into_mut()),
            linear_map::Entry::Vacant(vacant) => {
                tracing::debug!(asset_id = ?self.asset_id, backend_id = ?context.backend.id(), "loading material to gpu");
                let material_data = self.data.as_ref()?;
                let loaded = material_data.load(context);
                Some(vacant.insert(loaded))
            }
        }
    }

    pub fn from_diffuse_image(diffuse: impl Into<Arc<RgbaImage>>) -> Self {
        Self {
            asset_id: None,
            data: Some(MaterialData {
                diffuse: Some(diffuse.into()),
                ..Default::default()
            }),
            loaded: LinearMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LoadedMaterial {
    id: LoadedMaterialId,
    bind_group: Arc<ThreadLocalCell<wgpu::BindGroup>>,
}

impl LoadedMaterial {
    pub fn id(&self) -> LoadedMaterialId {
        self.id
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        self.bind_group.get()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoadedMaterialId(wgpu::Id<wgpu::BindGroup>);

#[derive(Debug, Default)]
struct MaterialData {
    ambient: Option<Arc<RgbaImage>>,
    diffuse: Option<Arc<RgbaImage>>,
    specular: Option<Arc<RgbaImage>>,
    normal: Option<Arc<RgbaImage>>,
    shininess: Option<Arc<RgbaImage>>,
    dissolve: Option<Arc<RgbaImage>>,
}

impl MaterialData {
    pub fn load(&self, context: &LoadContext) -> LoadedMaterial {
        let load_texture = |image: &RgbaImage| -> Texture {
            Texture::load(
                image,
                context.pipeline.default_sampler.clone(),
                &context.backend,
            )
        };

        let ambient = self.ambient.as_deref().map(load_texture);
        let diffuse = self.diffuse.as_deref().map(load_texture);
        let specular = self.specular.as_deref().map(load_texture);
        let normal = self.normal.as_deref().map(load_texture);
        let shininess = self.shininess.as_deref().map(load_texture);
        let dissolve = self.dissolve.as_deref().map(load_texture);

        fn texture_view_bind_group_entry<'a>(
            binding: u32,
            texture: &'a Option<Texture>,
            fallback: &'a Texture,
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
            texture: &'a Option<Texture>,
            fallback: &'a Texture,
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

        LoadedMaterial {
            id: LoadedMaterialId(bind_group.global_id()),
            bind_group: Arc::new(ThreadLocalCell::new(bind_group)),
        }
    }
}
