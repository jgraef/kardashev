use std::{
    fmt::Debug,
    future::Future,
    sync::Arc,
};

use bytes::Bytes;
use idb::DatabaseEvent;
use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};

use crate::utils::thread_local_cell::ThreadLocalCell;

#[derive(Clone, Debug)]
pub struct FileStore {
    database: Arc<idb::Database>,
}

impl FileStore {
    const VERSION: u32 = 1;

    pub fn delete(database_name: &str) -> Result<(), Error> {
        tracing::debug!(database_name, "deleting file store");

        let factory = idb::Factory::new()?;
        factory.delete(database_name)?;
        Ok(())
    }

    pub async fn open(database_name: &str) -> Result<Self, Error> {
        tracing::debug!(database_name, "opening file store");

        let factory = idb::Factory::new()?;

        let mut open_request = factory.open(database_name, Some(Self::VERSION))?;
        open_request.on_upgrade_needed(|event| {
            if let Err(error) = handle_upgrade_needed(event) {
                tracing::error!(?error, "error while upgrading database");
            }
        });

        let database = open_request.await?;

        Ok(Self {
            database: Arc::new(database),
        })
    }

    pub async fn get<M: DeserializeOwned>(
        &self,
        file_name: &str,
    ) -> Result<Option<File<M>>, Error> {
        tracing::debug!(file_name, "get file");

        let transaction = self
            .database
            .transaction(&["files"], idb::TransactionMode::ReadOnly)?;
        let store = transaction.object_store("files")?;

        if let Some(file) = store.get(idb::Query::Key(file_name.into()))?.await? {
            Ok(serde_wasm_bindgen::from_value(file)?)
        }
        else {
            Ok(None)
        }
    }

    pub async fn get_or_insert<M, F, I, Fut, E>(
        &self,
        file_name: &str,
        filter: F,
        insert: I,
    ) -> Result<File<M>, GetOrInsertError<E>>
    where
        M: Serialize + DeserializeOwned,
        F: FnOnce(&M) -> bool,
        I: FnOnce() -> Fut,
        Fut: Future<Output = Result<InsertFile<M>, E>>,
    {
        tracing::debug!(file_name, "get or insert file");

        let transaction = self
            .database
            .transaction(&["files"], idb::TransactionMode::ReadWrite)
            .map_err(Error::from)?;
        let store = transaction.object_store("files").map_err(Error::from)?;

        if let Some(file) = store
            .get(idb::Query::Key(file_name.into()))
            .map_err(Error::from)?
            .await
            .map_err(Error::from)?
        {
            let file: File<M> = serde_wasm_bindgen::from_value(file).map_err(Error::from)?;
            if filter(&file.meta_data) {
                transaction.await.map_err(Error::from)?;
                return Ok(file);
            }
        }

        let insert_file = insert()
            .await
            .map_err(|error| GetOrInsertError::Insert(error))?;
        let file = File {
            file_name: file_name.to_owned(),
            meta_data: insert_file.meta_data,
            data: insert_file.data.into(),
        };
        let js_value = serde_wasm_bindgen::to_value(&file).map_err(Error::from)?;
        store
            .put(&js_value, None)
            .map_err(Error::from)?
            .await
            .map_err(Error::from)?;
        transaction.await.map_err(Error::from)?;
        Ok(file)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("asset cache error: idb error: {message}")]
    Idb {
        message: String,
        error: ThreadLocalCell<idb::Error>,
    },
    #[error("asset cache error: serde_wasm_bindgen error: {message}")]
    SerdeWasmBindgen {
        message: String,
        error: ThreadLocalCell<serde_wasm_bindgen::Error>,
    },
}

impl From<idb::Error> for Error {
    fn from(error: idb::Error) -> Self {
        let message = error.to_string();
        let error = ThreadLocalCell::new(error);
        Self::Idb { message, error }
    }
}

impl From<serde_wasm_bindgen::Error> for Error {
    fn from(error: serde_wasm_bindgen::Error) -> Self {
        let message = error.to_string();
        let error = ThreadLocalCell::new(error);
        Self::SerdeWasmBindgen { message, error }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("asset cache get or insert error")]
pub enum GetOrInsertError<E> {
    Insert(#[source] E),
    FileStore(#[from] Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File<M> {
    pub file_name: String,
    pub meta_data: M,
    pub data: Bytes,
}

#[derive(Clone, Debug)]
pub struct InsertFile<M> {
    pub meta_data: M,
    pub data: Bytes,
}

fn handle_upgrade_needed(event: idb::event::VersionChangeEvent) -> Result<(), Error> {
    let database = event.database()?;

    let old_version = event.old_version()?;
    let new_version = event.new_version()?;
    tracing::debug!(old_version, ?new_version);

    let mut store_params = idb::ObjectStoreParams::new();
    store_params.key_path(Some(idb::KeyPath::new_single("file_name")));
    let _store = database.create_object_store("files", store_params)?;

    Ok(())
}
