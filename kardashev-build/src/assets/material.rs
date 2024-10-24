use std::collections::HashMap;

use kardashev_protocol::assets::AssetId;
use palette::Srgb;

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
        MaterialColorTexture,
        MaterialProperty,
        MaterialScalarTexture,
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

        async fn process_material_texture_asset<'a, 'b: 'a>(
            texture_asset_id: AssetId,
            context: &'a mut ProcessContext<'b>,
            freshness: &mut Freshness,
            material_asset_id: AssetId,
        ) -> Result<AssetId, Error> {
            let texture = context
                .source
                .get_asset::<Texture>(texture_asset_id)
                .ok_or_else(|| {
                    Error::AssetNotFound {
                        id: texture_asset_id,
                    }
                })?;
            texture.process(texture_asset_id, context).await?;
            freshness.and(context.source_asset(material_asset_id, texture_asset_id));
            Ok(texture_asset_id)
        }

        async fn process_material_texture_inline<'a, 'b: 'a>(
            texture: &Texture,
            context: &'a mut ProcessContext<'b>,
            freshness: &mut Freshness,
            material_asset_id: AssetId,
            property: MaterialProperty,
        ) -> Result<AssetId, Error> {
            let texture_asset_id =
                context
                    .build_info
                    .generate_id(GeneratedIdKey::MaterialTexture {
                        material: material_asset_id,
                        property,
                    });
            texture.process(texture_asset_id, context).await?;
            freshness.and(context.source_asset(material_asset_id, texture_asset_id));
            Ok(texture_asset_id)
        }

        async fn process_material_texture_asset_or_inline<'a, 'b: 'a>(
            texture: &AssetIdOrInline<Texture>,
            context: &'a mut ProcessContext<'b>,
            freshness: &mut Freshness,
            material_asset_id: AssetId,
            property: MaterialProperty,
        ) -> Result<AssetId, Error> {
            match texture {
                AssetIdOrInline::AssetId(texture_asset_id) => {
                    process_material_texture_asset(
                        *texture_asset_id,
                        context,
                        freshness,
                        material_asset_id,
                    )
                    .await
                }
                AssetIdOrInline::Inline(texture) => {
                    process_material_texture_inline(
                        texture,
                        context,
                        freshness,
                        material_asset_id,
                        property,
                    )
                    .await
                }
            }
        }

        async fn process_texture<'a, 'b: 'a>(
            texture: &Option<AssetIdOrInline<Texture>>,
            context: &'a mut ProcessContext<'b>,
            freshness: &mut Freshness,
            material_asset_id: AssetId,
            property: MaterialProperty,
        ) -> Result<Option<AssetId>, Error> {
            if let Some(texture) = texture {
                process_material_texture_asset_or_inline(
                    texture,
                    context,
                    freshness,
                    material_asset_id,
                    property,
                )
                .await
                .map(Some)
            }
            else {
                Ok(None)
            }
        }

        async fn process_material_color_texture<'a, 'b: 'a>(
            input: &Option<MaterialColorTexture>,
            context: &'a mut ProcessContext<'b>,
            freshness: &mut Freshness,
            material_asset_id: AssetId,
            property: MaterialProperty,
        ) -> Result<(Option<AssetId>, Option<Srgb<f32>>), Error> {
            if let Some(input) = input {
                let texture = process_texture(
                    &input.texture,
                    context,
                    freshness,
                    material_asset_id,
                    property,
                )
                .await?;
                Ok((texture, input.tint))
            }
            else {
                Ok((None, None))
            }
        }

        async fn process_material_scalar_texture<'a, 'b: 'a>(
            input: &Option<MaterialScalarTexture>,
            context: &'a mut ProcessContext<'b>,
            freshness: &mut Freshness,
            material_asset_id: AssetId,
            property: MaterialProperty,
        ) -> Result<(Option<AssetId>, Option<f32>), Error> {
            if let Some(input) = input {
                let texture = process_texture(
                    &input.texture,
                    context,
                    freshness,
                    material_asset_id,
                    property,
                )
                .await?;
                Ok((texture, input.value))
            }
            else {
                Ok((None, None))
            }
        }

        let normal_texture = process_texture(
            &self.normal,
            context,
            &mut freshness,
            id,
            MaterialProperty::Normal,
        )
        .await?;
        let (ambient_texture, ambient_color) = process_material_color_texture(
            &self.ambient,
            context,
            &mut freshness,
            id,
            MaterialProperty::Ambient,
        )
        .await?;
        let (diffuse_texture, diffuse_color) = process_material_color_texture(
            &self.diffuse,
            context,
            &mut freshness,
            id,
            MaterialProperty::Diffuse,
        )
        .await?;
        let (specular_texture, specular_color) = process_material_color_texture(
            &self.specular,
            context,
            &mut freshness,
            id,
            MaterialProperty::Specular,
        )
        .await?;
        let (shininess_texture, shininess) = process_material_scalar_texture(
            &self.shininess,
            context,
            &mut freshness,
            id,
            MaterialProperty::Shininess,
        )
        .await?;
        let (dissolve_texture, dissolve) = process_material_scalar_texture(
            &self.dissolve,
            context,
            &mut freshness,
            id,
            MaterialProperty::Dissolve,
        )
        .await?;
        let (emissive_texture, emissive_color) = process_material_color_texture(
            &self.emissive,
            context,
            &mut freshness,
            id,
            MaterialProperty::Emissive,
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
            normal_texture,
            ambient_texture,
            ambient_color,
            diffuse_texture,
            diffuse_color,
            specular_texture,
            specular_color,
            shininess_texture,
            shininess,
            dissolve_texture,
            dissolve,
            emissive_texture,
            emissive_color,
            albedo_texture: None,
            metalness_texture: None,
            roughness_texture: None,
        });

        context.set_build_time(id);

        Ok(())
    }
}
