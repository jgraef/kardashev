use std::collections::{
    HashMap,
    HashSet,
};

use kardashev_protocol::assets::AssetId;
use serde::{
    Deserialize,
    Serialize,
};
use uuid::Uuid;

use crate::processor::material::MaterialProperty;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(from = "serialize::BuildInfo", into = "serialize::BuildInfo")]
pub struct BuildInfo {
    pub dependencies: HashMap<AssetId, HashSet<AssetId>>,
    pub generated_ids: HashMap<GeneratedIdKey, AssetId>,
}

impl BuildInfo {
    pub fn generate_id(&mut self, key: GeneratedIdKey) -> AssetId {
        *self
            .generated_ids
            .entry(key)
            .or_insert_with(|| AssetId(Uuid::new_v4()))
    }

    pub fn add_dependency(&mut self, id: AssetId, depends_on: AssetId) {
        self.dependencies.entry(id).or_default().insert(depends_on);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GeneratedIdKey {
    MaterialTexture {
        material: AssetId,
        property: MaterialProperty,
    },
}

mod serialize {
    use kardashev_protocol::assets::AssetId;
    use serde::{
        Deserialize,
        Serialize,
    };

    use crate::build_info::GeneratedIdKey;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct BuildInfo {
        dependencies: Vec<Dependency>,
        generated_ids: Vec<GeneratedId>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Dependency {
        id: AssetId,
        depends_on: Vec<AssetId>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct GeneratedId {
        key: GeneratedIdKey,
        id: AssetId,
    }

    impl From<BuildInfo> for super::BuildInfo {
        fn from(value: BuildInfo) -> Self {
            Self {
                dependencies: value
                    .dependencies
                    .into_iter()
                    .map(|dependency| (dependency.id, dependency.depends_on.into_iter().collect()))
                    .collect(),
                generated_ids: value
                    .generated_ids
                    .into_iter()
                    .map(|generated_id| (generated_id.key, generated_id.id))
                    .collect(),
            }
        }
    }

    impl From<super::BuildInfo> for BuildInfo {
        fn from(value: super::BuildInfo) -> Self {
            Self {
                dependencies: value
                    .dependencies
                    .into_iter()
                    .map(|(id, depends_on)| {
                        Dependency {
                            id,
                            depends_on: depends_on.into_iter().collect(),
                        }
                    })
                    .collect(),
                generated_ids: value
                    .generated_ids
                    .into_iter()
                    .map(|(key, id)| GeneratedId { key, id })
                    .collect(),
            }
        }
    }
}
