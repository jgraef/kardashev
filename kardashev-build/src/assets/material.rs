use std::collections::HashMap;

use kardashev_protocol::assets::AssetId;

use crate::assets::{
    build_info::GeneratedIdKey,
    dist,
    processor::{
        Freshness,
        ProcessContext,
    },
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

    async fn process<'a, 'b: 'a>(
        &'a self,
        id: AssetId,
        context: &'a mut ProcessContext<'b>,
    ) -> Result<(), Error> {
        if !context.processing(id) {
            return Ok(());
        }

        let mut freshness = Freshness::Fresh;

        async fn process_texture<'a, 'b: 'a>(
            texture: &Option<AssetIdOrInline<Texture>>,
            property: MaterialProperty,
            id: AssetId,
            mut context: &'a mut ProcessContext<'b>,
            freshness: &mut Freshness,
        ) -> Result<Option<AssetId>, Error> {
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

                texture.process(texture_asset_id, &mut context).await?;
                freshness.and(context.source_asset(id, texture_asset_id));

                Ok(Some(texture_asset_id))
            }
            else {
                Ok(None)
            }
        }

        let normal = process_texture(
            &self.normal,
            MaterialProperty::Normal,
            id,
            context,
            &mut freshness,
        )
        .await?;
        let ambient = process_texture(
            &self.ambient,
            MaterialProperty::Ambient,
            id,
            context,
            &mut freshness,
        )
        .await?;
        let diffuse = process_texture(
            &self.diffuse,
            MaterialProperty::Diffuse,
            id,
            context,
            &mut freshness,
        )
        .await?;
        let specular = process_texture(
            &self.specular,
            MaterialProperty::Specular,
            id,
            context,
            &mut freshness,
        )
        .await?;
        let shininess = process_texture(
            &self.shininess,
            MaterialProperty::Shininess,
            id,
            context,
            &mut freshness,
        )
        .await?;
        let dissolve = process_texture(
            &self.dissolve,
            MaterialProperty::Dissolve,
            id,
            context,
            &mut freshness,
        )
        .await?;

        let albedo = process_texture(
            &self.albedo,
            MaterialProperty::Albedo,
            id,
            context,
            &mut freshness,
        )
        .await?;
        let metalness = process_texture(
            &self.metalness,
            MaterialProperty::Metalness,
            id,
            context,
            &mut freshness,
        )
        .await?;
        let roughness = process_texture(
            &self.roughness,
            MaterialProperty::Roughness,
            id,
            context,
            &mut freshness,
        )
        .await?;

        if freshness.is_fresh() {
            tracing::debug!(%id, "not modified since last build. skipping.");
            return Ok(());
        }

        context.dist_assets.insert(dist::Material {
            id,
            label: self.label.clone(),
            build_time: context.build_time,
            normal,
            ambient,
            diffuse,
            specular,
            shininess,
            dissolve,
            albedo,
            metalness,
            roughness,
        });

        context.set_build_time(id);

        Ok(())
    }
}
