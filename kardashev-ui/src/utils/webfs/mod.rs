mod database;
mod fs_impl;
pub mod path;

use std::sync::Arc;

use tokio::sync::{
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};

use crate::utils::{
    thread_local_cell::ThreadLocalCell,
    webfs::database::Database,
};

pub async fn open() -> Result<WebFs, Error> {
    static SINGLETON: RwLock<Option<WebFs>> = RwLock::const_new(None);
    const DATABASE_NAME: &'static str = "webfs";

    let mut singleton = SINGLETON.write().await;

    let webfs = if let Some(webfs) = singleton.as_ref() {
        webfs.clone()
    }
    else {
        let webfs = WebFs::open(DATABASE_NAME).await?;
        *singleton = Some(webfs.clone());
        webfs
    };

    Ok(webfs)
}

#[derive(Clone, Debug)]
pub struct WebFs {
    database: Arc<RwLock<ThreadLocalCell<Database>>>,
}

impl WebFs {
    async fn open(database_name: &str) -> Result<Self, Error> {
        let database = Database::open(database_name).await?;

        Ok(Self {
            database: Arc::new(RwLock::new(ThreadLocalCell::new(database))),
        })
    }

    pub async fn read(&self) -> ReadTransaction {
        let inner = self.database.read().await;
        ReadTransaction { inner }
    }

    pub async fn write(&self) -> WriteTransaction {
        let inner = self.database.write().await;
        WriteTransaction { inner }
    }
}

#[derive(Debug)]
pub struct ReadTransaction<'a> {
    inner: RwLockReadGuard<'a, ThreadLocalCell<Database>>,
}

#[derive(Debug)]
pub struct WriteTransaction<'a> {
    inner: RwLockWriteGuard<'a, ThreadLocalCell<Database>>,
}

#[derive(Debug, thiserror::Error)]
#[error("webfs error")]
pub struct Error {
    #[from]
    implementation: database::Error,
}
