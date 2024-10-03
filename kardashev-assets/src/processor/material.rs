use std::path::Path;

use kardashev_protocol::assets::AssetId;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    build_info::GeneratedIdKey,
    dist,
    processor::{
        Process,
        Processor,
    },
    source::{
        AssetIdOrInline,
        Material,
        Texture,
    },
    Error,
};

impl Process for Material {
    fn process(
        &self,
        id: AssetId,
        mut processor: &mut Processor,
        manifest_path: &Path,
    ) -> Result<(), Error> {
        let mut process_texture = |texture: &Option<AssetIdOrInline<Texture>>,
                                   property: MaterialProperty|
         -> Result<Option<AssetId>, Error> {
            if let Some(texture) = texture {
                let texture_asset_id = match texture {
                    AssetIdOrInline::AssetId(asset_id) => *asset_id,
                    AssetIdOrInline::Inline(texture) => {
                        let texture_asset_id =
                            processor
                                .build_info
                                .generate_id(GeneratedIdKey::MaterialTexture {
                                    material: id,
                                    property,
                                });

                        texture.process(texture_asset_id, &mut processor, manifest_path)?;

                        texture_asset_id
                    }
                };

                processor.build_info.add_dependency(id, texture_asset_id);

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

        processor.dist_manifest.materials.push(dist::Material {
            id,
            label: self.label.clone(),
            ambient,
            diffuse,
            specular,
            normal,
            shininess,
            dissolve,
        });

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialProperty {
    Ambient,
    Diffuse,
    Specular,
    Normal,
    Shininess,
    Dissolve,
}
