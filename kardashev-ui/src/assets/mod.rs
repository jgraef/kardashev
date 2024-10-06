mod dyn_type;
pub mod image_load;
pub mod load;
mod server;
pub mod system;

use std::fmt::Debug;

use kardashev_protocol::assets::AssetId;

#[derive(Debug, thiserror::Error)]
#[error("asset loader error")]
pub enum Error {
    Reqwest(#[from] reqwest::Error),
    AssetNotFound(#[from] AssetNotFound),
    ImageLoad(#[from] image_load::LoadImageError),
    Graphics(#[from] crate::graphics::Error),
    Client(#[from] kardashev_client::Error),
    AssetParse(#[from] kardashev_protocol::assets::AssetParseError),
}

#[derive(Debug, thiserror::Error)]
#[error("asset not found: {asset_id}")]
pub struct AssetNotFound {
    pub asset_id: AssetId,
}

pub trait MaybeHasAssetId {
    fn maybe_asset_id(&self) -> Option<AssetId>;
}
