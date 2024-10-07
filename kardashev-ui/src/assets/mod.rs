mod dyn_type;
pub mod image_load;
pub mod load;
mod server;
pub mod system;

use std::fmt::Debug;

use chrono::{
    DateTime,
    Utc,
};
use kardashev_protocol::assets::AssetId;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, thiserror::Error)]
#[error("asset loader error")]
pub enum Error {
    Reqwest(#[from] reqwest::Error),
    AssetNotFound(#[from] AssetNotFound),
    ImageLoad(#[from] image_load::LoadImageError),
    Graphics(#[from] crate::graphics::Error),
    Client(#[from] kardashev_client::Error),
    AssetParse(#[from] kardashev_protocol::assets::AssetParseError),
    FileStore(#[from] crate::utils::file_store::Error),
}

#[derive(Debug, thiserror::Error)]
#[error("asset not found: {asset_id}")]
pub struct AssetNotFound {
    pub asset_id: AssetId,
}

pub trait MaybeHasAssetId {
    fn maybe_asset_id(&self) -> Option<AssetId>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileCacheMetaData {
    pub asset_id: AssetId,
    pub build_time: DateTime<Utc>,
}
