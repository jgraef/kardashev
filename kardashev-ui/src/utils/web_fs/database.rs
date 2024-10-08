use std::{
    marker::PhantomData,
    sync::Arc,
};

use bitflags::bitflags;
use idb::DatabaseEvent;
use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};

use crate::utils::thread_local_cell::ThreadLocalCell;

const INODES_STORE: &'static str = "inodes";
const BLOBS_STORE: &'static str = "blobs";

#[derive(Clone, Debug)]
pub struct Database {
    database: Arc<ThreadLocalCell<idb::Database>>,
}

impl Database {
    const VERSION: u32 = 1;

    pub async fn open(database_name: &str) -> Result<Self, Error> {
        fn handle_upgrade_needed(event: idb::event::VersionChangeEvent) -> Result<(), Error> {
            let database = event.database()?;

            let old_version = event.old_version()?;
            let new_version = event.new_version()?;
            tracing::debug!(old_version, ?new_version);

            let mut store_params = idb::ObjectStoreParams::new();
            store_params
                .key_path(Some(idb::KeyPath::new_single("id")))
                .auto_increment(true);
            let inodes_store = database.create_object_store("inodes", store_params)?;

            let index_params = idb::IndexParams::new();
            let _parent_index = inodes_store.create_index(
                "parent",
                idb::KeyPath::new_single("parent"),
                Some(index_params),
            )?;

            let mut index_params = idb::IndexParams::new();
            index_params.unique(true);
            let _file_name_index = inodes_store.create_index(
                "file_name",
                idb::KeyPath::new_array(["parent", "file_name"]),
                Some(index_params),
            )?;

            let mut store_params = idb::ObjectStoreParams::new();
            store_params
                .key_path(Some(idb::KeyPath::new_single("id")))
                .auto_increment(true);
            let _store = database.create_object_store("blobs", store_params)?;

            Ok(())
        }

        tracing::debug!(database_name, "opening webfs");

        let factory = idb::Factory::new()?;

        let mut open_request = factory.open(database_name, Some(Self::VERSION))?;
        open_request.on_upgrade_needed(|event| {
            if let Err(error) = handle_upgrade_needed(event) {
                tracing::error!(?error, "error while upgrading database");
            }
        });

        let database = open_request.await?;

        Ok(Self {
            database: Arc::new(ThreadLocalCell::new(database)),
        })
    }

    pub fn transaction(
        &self,
        scope: Scope,
        mode: idb::TransactionMode,
    ) -> Result<Transaction, Error> {
        let transaction = self.database.get().transaction(scope.names(), mode)?;
        Ok(Transaction {
            transaction,
            _lt: PhantomData,
        })
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct Scope: u8 {
        const INODES = 0b01;
        const BLOBS = 0b10;
        const ALL = 0b11;
    }
}

impl Scope {
    fn names(self) -> &'static [&'static str] {
        match self {
            Self::INODES => &[INODES_STORE],
            Self::BLOBS => &[BLOBS_STORE],
            Self::ALL => &[INODES_STORE, BLOBS_STORE],
            _ => panic!("invalid scope"),
        }
    }
}

#[derive(Debug)]
pub struct Transaction<'t> {
    transaction: idb::Transaction,
    _lt: PhantomData<&'t ()>,
}

impl<'t> Transaction<'t> {
    pub fn commit(self) -> Result<(), Error> {
        self.transaction.commit()?;
        Ok(())
    }

    pub async fn get_inode<M: DeserializeOwned>(
        &self,
        inode_id: InodeId,
    ) -> Result<Option<GetInode<M>>, Error> {
        let inodes_store = self.transaction.object_store("inodes")?;
        let query = serde_wasm_bindgen::to_value(&inode_id)?;

        if let Some(value) = inodes_store.get(query)?.await? {
            let inode: GetInode<M> = serde_wasm_bindgen::from_value(value)?;
            Ok(Some(inode))
        }
        else {
            Ok(None)
        }
    }

    pub async fn get_inode_by_name<'a, M: DeserializeOwned>(
        &self,
        file_name: &str,
        parent: Option<InodeId>,
    ) -> Result<Option<GetInode<M>>, Error> {
        let inodes_store = self.transaction.object_store("inodes")?;
        let file_name_index = inodes_store.index("file_name")?;
        let query = serde_wasm_bindgen::to_value(&QueryInodeByName { file_name, parent })?;

        if let Some(value) = file_name_index.get(query)?.await? {
            let inode: GetInode<M> = serde_wasm_bindgen::from_value(value)?;
            Ok(Some(inode))
        }
        else {
            Ok(None)
        }
    }

    pub async fn get_inodes<M: DeserializeOwned>(
        &self,
        parent: Option<InodeId>,
    ) -> Result<Vec<GetInode<M>>, Error> {
        let inodes_store = self.transaction.object_store("inodes")?;
        let parent_index = inodes_store.index("parent")?;
        let query = serde_wasm_bindgen::to_value(&QueryInodes { parent })?;

        let inodes = parent_index
            .get_all(Some(idb::Query::Key(query)), None)?
            .await?
            .into_iter()
            .map(|value| serde_wasm_bindgen::from_value(value))
            .collect::<Result<Vec<GetInode<M>>, _>>()?;

        Ok(inodes)
    }

    pub async fn insert_inode<'a, M: Serialize>(
        &self,
        inode: &InsertInode<'a, M>,
    ) -> Result<InodeId, Error> {
        let inodes_store = self.transaction.object_store("inodes")?;
        let value = serde_wasm_bindgen::to_value(inode)?;
        let value = inodes_store.put(&value, None)?.await?;
        let inode_id = serde_wasm_bindgen::from_value(value)?;
        Ok(inode_id)
    }

    pub async fn get_blob(&self, blob_id: BlobId) -> Result<Option<GetBlob>, Error> {
        let blobs_store = self.transaction.object_store("blobs")?;
        let query = serde_wasm_bindgen::to_value(&blob_id)?;

        if let Some(value) = blobs_store.get(query)?.await? {
            let blob = serde_wasm_bindgen::from_value(value)?;
            Ok(Some(blob))
        }
        else {
            Ok(None)
        }
    }

    pub async fn insert_blob(&self, blob: &InsertBlob) -> Result<BlobId, Error> {
        let blobs_store = self.transaction.object_store("blobs")?;
        let value = serde_wasm_bindgen::to_value(blob)?;
        let value = blobs_store.put(&value, None)?.await?;
        let blob_id = serde_wasm_bindgen::from_value(value)?;
        Ok(blob_id)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct InodeId(u32);

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct BlobId(u32);

#[derive(Debug, Serialize)]
struct QueryInodeByName<'a> {
    pub file_name: &'a str,
    pub parent: Option<InodeId>,
}

#[derive(Debug, Serialize)]
struct QueryInodes {
    pub parent: Option<InodeId>,
}

#[derive(Clone, Debug, Serialize)]
pub struct InsertInode<'a, M> {
    pub id: Option<InodeId>,
    pub parent: Option<InodeId>,
    pub file_name: &'a str,
    pub meta_data: &'a M,
    pub kind: &'a InodeKind,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetInode<M> {
    pub id: InodeId,
    pub parent: Option<InodeId>,
    pub file_name: String,
    pub meta_data: M,
    pub kind: InodeKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InodeKind {
    File { blob_id: Option<BlobId> },
    Directory,
}

#[derive(Debug, Deserialize)]
pub struct GetBlob {
    pub id: BlobId,
    #[serde(with = "serde_wasm_bindgen::preserve")]
    pub data: web_sys::Blob,
}

#[derive(Debug, Serialize)]
pub struct InsertBlob {
    pub id: Option<BlobId>,

    #[serde(with = "serde_wasm_bindgen::preserve")]
    pub data: web_sys::Blob,
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
