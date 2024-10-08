use std::{
    collections::HashMap,
    sync::Arc,
};

use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};

use crate::utils::webfs::{
    database::{
        self,
        Database,
        GetInode,
        InodeKind,
        InsertInode, Scope,
    },
    path::PathBuf,
};

#[derive(Debug)]
pub struct WebFs {
    database: Arc<Database>,
    root: GetInode<InodeMetadata>,
    current: GetInode<InodeMetadata>,
}

impl WebFs {
    pub async fn open_named_root(database: Database, root: &str) -> Result<Self, Error> {
        let transaction = database.transaction(Scope::INODES, idb::TransactionMode::ReadOnly)?;
        let root = if let Some(root) = transaction.get_inode_by_name(root, None).await? {
            root
        }
        else {
            let meta_data = InodeMetadata::default();
            let inode_id = transaction
                .insert_inode(&InsertInode {
                    parent: None,
                    file_name: root,
                    meta_data: &meta_data,
                    kind: InodeKind::Directory,
                })
                .await?;
            GetInode {
                id: inode_id,
                parent: None,
                file_name: root.to_owned(),
                meta_data,
                kind: InodeKind::Directory,
            }
        };

        Ok(Self::new(database, root))
    }

    pub fn new(database: Database, root: GetInode<InodeMetadata>) -> Self {
        Self {
            database: Arc::new(database),
            root: root.clone(),
            current: root,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InodeMetadata {
    meta_data: HashMap<String, serde_json::Value>,
}

impl InodeMetadata {
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, serde_json::Error> {
        if let Some(value) = self.meta_data.get(key) {
            let value = T::deserialize(value)?;
            Ok(Some(value))
        }
        else {
            Ok(None)
        }
    }

    pub fn insert<T: Serialize>(
        &mut self,
        key: String,
        value: &T,
    ) -> Result<(), serde_json::Error> {
        let value = serde_json::to_value(value)?;
        self.meta_data.insert(key, value);
        Ok(())
    }
}

pub struct File {}

#[derive(Debug, thiserror::Error)]
#[error("webfs error")]
pub enum Error {
    Database(#[from] database::Error),
    InvalidPath { path: PathBuf },
}
