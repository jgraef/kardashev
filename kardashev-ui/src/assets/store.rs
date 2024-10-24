use std::ops::Deref;

use chrono::{
    DateTime,
    Utc,
};
use kardashev_protocol::assets::AssetId;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    assets::Error,
    utils::web_fs::{
        file_lock::FileLockWriteGuard,
        File,
        OpenOptions,
        WebFs,
    },
};

#[derive(Debug)]
pub struct AssetStore {
    web_fs: WebFs,
    lock_file: File,
}

impl AssetStore {
    pub(super) async fn new() -> Result<Self, Error> {
        let web_fs = WebFs::with_named_root("assets").await?;
        let lock_file = web_fs
            .open(".lock", OpenOptions::new().create(true))
            .await?;
        Ok(Self { web_fs, lock_file })
    }

    pub async fn lock(&self) -> AssetStoreGuard {
        let guard = self.lock_file.lock_write().await;
        AssetStoreGuard {
            web_fs: self.web_fs.clone(),
            _guard: guard,
        }
    }
}

#[derive(Debug)]
pub struct AssetStoreGuard {
    web_fs: WebFs,
    _guard: FileLockWriteGuard,
}

impl Deref for AssetStoreGuard {
    type Target = WebFs;

    fn deref(&self) -> &Self::Target {
        &self.web_fs
    }
}

impl AsRef<WebFs> for AssetStoreGuard {
    fn as_ref(&self) -> &WebFs {
        &self.web_fs
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AssetStoreMetaData {
    pub asset_id: Option<AssetId>,
    pub build_time: Option<DateTime<Utc>>,
}
