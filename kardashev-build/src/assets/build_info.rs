use std::collections::HashMap;

use chrono::{
    DateTime,
    Utc,
};
use kardashev_protocol::assets::AssetId;
use serde::{
    Deserialize,
    Serialize,
};

use crate::assets::source::MaterialProperty;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BuildInfo {
    pub generated_ids: GeneratedIds,
    pub build_times: HashMap<AssetId, DateTime<Utc>>,
}

impl BuildInfo {
    pub fn generate_id(&mut self, key: GeneratedIdKey) -> AssetId {
        self.generated_ids.generate_id(key)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(from = "serialize::GeneratedIds", into = "serialize::GeneratedIds")]
pub struct GeneratedIds {
    inner: HashMap<GeneratedIdKey, AssetId>,
}

impl GeneratedIds {
    pub fn generate_id(&mut self, key: GeneratedIdKey) -> AssetId {
        *self.inner.entry(key).or_insert_with(|| AssetId::generate())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GeneratedIdKey {
    MaterialTexture {
        material: AssetId,
        property: MaterialProperty,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Precompressed {
    pub format: CompressionFormat,
    pub compressed: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompressionFormat {
    Gzip,
}

mod serialize {
    use kardashev_protocol::assets::AssetId;
    use serde::{
        Deserialize,
        Serialize,
    };

    use super::GeneratedIdKey;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct GeneratedIds {
        inner: Vec<GeneratedId>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct GeneratedId {
        key: GeneratedIdKey,
        id: AssetId,
    }

    impl From<GeneratedIds> for super::GeneratedIds {
        fn from(value: GeneratedIds) -> Self {
            Self {
                inner: value
                    .inner
                    .into_iter()
                    .map(|generated_id| (generated_id.key, generated_id.id))
                    .collect(),
            }
        }
    }

    impl From<super::GeneratedIds> for GeneratedIds {
        fn from(value: super::GeneratedIds) -> Self {
            Self {
                inner: value
                    .inner
                    .into_iter()
                    .map(|(key, id)| GeneratedId { key, id })
                    .collect(),
            }
        }
    }
}
