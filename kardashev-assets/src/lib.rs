pub mod atlas;
pub mod build_info;
mod material;
mod mesh;
pub mod processor;
#[cfg(feature = "server")]
pub mod server;
mod shader;
pub mod source;
mod texture;

use std::{
    collections::HashMap,
    path::Path,
};

pub use kardashev_protocol::assets::{
    self as dist,
    AssetId,
};

use crate::{
    processor::{
        ProcessContext,
        Processed,
        Processor,
    },
    source::Manifest,
};

pub trait Asset: Sized + 'static {
    fn register_dist_type(dist_asset_types: &mut dist::AssetTypes);

    fn get_assets(manifest: &Manifest) -> &HashMap<AssetId, Self>;

    fn process<'a, 'b: 'a>(
        &self,
        id: AssetId,
        context: &'a mut ProcessContext<'b>,
    ) -> Result<(), Error>;
}

#[derive(Debug, thiserror::Error)]
#[error("asset processing error")]
pub enum Error {
    AssetNotFound {
        id: AssetId,
    },
    Io(#[from] std::io::Error),
    Image(#[from] image::ImageError),
    MessagePackDecode(#[from] rmp_serde::decode::Error),
    MessagePackEncode(#[from] rmp_serde::encode::Error),
    Json(#[from] serde_json::Error),
    TomlDecode(#[from] toml::de::Error),
    WalkDir(#[from] walkdir::Error),
    WgslParse(#[from] naga::front::wgsl::ParseError),
    #[cfg(feature = "server")]
    Axum(#[from] axum::Error),
    #[cfg(feature = "server")]
    Notify(#[from] notify_async::Error),
    AssetParse(#[from] kardashev_protocol::assets::AssetParseError),
}

pub fn process(
    source_path: impl AsRef<Path>,
    dist_path: impl AsRef<Path>,
) -> Result<Processed, Error> {
    let mut processor = Processor::new(dist_path)?;
    processor.add_directory(source_path)?;
    processor.process()
}
