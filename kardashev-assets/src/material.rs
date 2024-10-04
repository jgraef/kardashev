use std::collections::HashMap;

use kardashev_protocol::assets::AssetId;

use crate::{
    build_info::GeneratedIdKey,
    dist,
    processor::ProcessContext,
    source::{
        AssetIdOrInline,
        Manifest,
        Material,
        MaterialProperty,
        Texture,
    },
    Asset,
    Error,
};

impl Asset for Material {
    fn register_dist_type(dist_asset_types: &mut dist::AssetTypes) {
        dist_asset_types.register::<dist::Material>();
    }

    fn get_assets(manifest: &Manifest) -> &HashMap<AssetId, Self> {
        &manifest.materials
    }

    fn process<'a, 'b: 'a>(
        &self,
        id: AssetId,
        mut context: &'a mut ProcessContext<'b>,
    ) -> Result<(), Error> {
        if !context.processing(id) {
            return Ok(());
        }

        let mut is_fresh = true;

        let mut process_texture = |texture: &Option<AssetIdOrInline<Texture>>,
                                   property: MaterialProperty|
         -> Result<Option<AssetId>, Error> {
            if let Some(texture) = texture {
                let (texture_asset_id, texture) = match texture {
                    AssetIdOrInline::AssetId(texture_asset_id) => {
                        let texture = context
                            .source
                            .get_asset::<Texture>(*texture_asset_id)
                            .ok_or_else(|| {
                                Error::AssetNotFound {
                                    id: *texture_asset_id,
                                }
                            })?;
                        (*texture_asset_id, texture)
                    }
                    AssetIdOrInline::Inline(texture) => {
                        let texture_asset_id =
                            context
                                .build_info
                                .generate_id(GeneratedIdKey::MaterialTexture {
                                    material: id,
                                    property,
                                });
                        (texture_asset_id, texture)
                    }
                };

                texture.process(texture_asset_id, &mut context)?;
                is_fresh &= context.is_fresh_dependency(id, texture_asset_id);

                Ok(Some(texture_asset_id))
            }
            else {
                Ok(None)
            }
        };

        let ambient = process_texture(&self.ambient, MaterialProperty::Ambient)?;
        let diffuse = process_texture(&self.diffuse, MaterialProperty::Diffuse)?;
        let specular = process_texture(&self.specular, MaterialProperty::Specular)?;
        let normal = process_texture(&self.normal, MaterialProperty::Normal)?;
        let shininess = process_texture(&self.shininess, MaterialProperty::Shininess)?;
        let dissolve = process_texture(&self.dissolve, MaterialProperty::Dissolve)?;

        if is_fresh {
            tracing::debug!(%id, "not modified since last build. skipping.");
            return Ok(());
        }

        context.dist_assets.insert(dist::Material {
            id,
            label: self.label.clone(),
            ambient,
            diffuse,
            specular,
            normal,
            shininess,
            dissolve,
        });

        context.set_build_time(id);

        Ok(())
    }
}
