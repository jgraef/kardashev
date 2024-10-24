pub mod atlas;
pub mod build_info;
mod material;
mod mesh;
pub mod processor;
mod shader;
pub mod source;
mod texture;

use std::{
    collections::HashMap,
    future::Future,
    path::Path,
};

pub use kardashev_protocol::assets::{
    self as dist,
    AssetId,
};

use crate::assets::{
    processor::{
        ProcessContext,
        Processed,
        Processor,
    },
    source::Manifest,
};

pub trait Asset: Sized + Send + Sync + 'static {
    fn register_dist_type(dist_asset_types: &mut dist::AssetTypes);

    fn get_assets(manifest: &Manifest) -> &HashMap<AssetId, Self>;

    fn process<'a, 'b: 'a>(
        &'a self,
        id: AssetId,
        context: &'a mut ProcessContext<'b>,
    ) -> impl Future<Output = Result<(), Error>> + Send + Sync + 'a;
}

#[derive(Debug, thiserror::Error)]
#[error("asset processing error")]
pub enum Error {
    #[error("asset not found: {id}")]
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
    Watch(#[from] crate::util::watch::Error),
    AssetParse(#[from] kardashev_protocol::assets::AssetParseError),
    NagaValidatation(#[from] naga::WithSpan<naga::valid::ValidationError>),
    InvalidColorName(#[from] crate::assets::source::InvalidColorName),
}

pub async fn process(
    source_path: impl AsRef<Path>,
    dist_path: impl AsRef<Path>,
    clean: bool,
) -> Result<Processed, Error> {
    let mut processor = Processor::new(dist_path)?;
    processor.add_directory(source_path)?;
    processor.process(clean).await
}
