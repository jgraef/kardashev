mod database;
pub mod file_lock;
pub mod path;

use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::Debug,
    sync::Arc,
};

use bitflags::bitflags;
use bytes::{
    Bytes,
    BytesMut,
};
use gloo_file::Blob;
use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};
use tokio::sync::{
    Mutex,
    RwLock,
};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use wasm_streams::ReadableStream;

use self::{
    database::{
        Database,
        GetInode,
        InodeKind,
        InsertBlob,
        InsertInode,
        Scope,
        Transaction,
    },
    file_lock::{
        FileLockReadGuard,
        FileLockWriteGuard,
        FileLocks,
    },
    path::{
        Component,
        Components,
        Path,
        PathBuf,
    },
};

#[derive(Clone, Debug)]
pub struct WebFs {
    database: Database,
    locks: FileLocks,
    state: Arc<State>,
}

#[derive(Debug)]
struct State {
    root_directory: GetInode<Metadata>,
    current_directory: RwLock<GetInode<Metadata>>,
}

impl WebFs {
    const DATABASE_NAME: &'static str = "web_fs";
    const DEFAULT_ROOT: &'static str = "default";

    pub async fn new() -> Result<Self, Error> {
        Self::with_named_root(Self::DEFAULT_ROOT).await
    }

    pub async fn with_named_root(root: &str) -> Result<Self, Error> {
        tracing::info!(root, "opening webfs");

        #[derive(Clone)]
        struct Singleton {
            database: Database,
            locks: FileLocks,
        }

        static SINGLETON: Mutex<Option<Singleton>> = Mutex::const_new(None);

        let mut singleton_guard = SINGLETON.lock().await;
        let singleton = if let Some(singleton) = &*singleton_guard {
            singleton.clone()
        }
        else {
            let singleton = Singleton {
                database: Database::open(Self::DATABASE_NAME).await?,
                locks: FileLocks::new(),
            };
            *singleton_guard = Some(singleton.clone());
            singleton
        };

        let transaction = singleton
            .database
            .transaction(Scope::INODES, idb::TransactionMode::ReadWrite)?;
        let root_directory =
            if let Some(root_directory) = transaction.get_inode_by_name(root, None).await? {
                root_directory
            }
            else {
                tracing::debug!("creating root directory");

                let meta_data = Metadata::default();
                let inode_id = transaction
                    .insert_inode(&InsertInode {
                        id: None,
                        parent: None,
                        file_name: root,
                        meta_data: &meta_data,
                        kind: &InodeKind::Directory,
                    })
                    .await?;

                tracing::trace!(root_inode_id = ?inode_id);

                GetInode {
                    id: inode_id,
                    parent: None,
                    file_name: root.to_owned(),
                    meta_data,
                    kind: InodeKind::Directory,
                }
            };

        transaction.commit()?;

        Ok(Self {
            database: singleton.database,
            locks: singleton.locks,
            state: Arc::new(State {
                root_directory: root_directory.clone(),
                current_directory: RwLock::new(root_directory),
            }),
        })
    }

    pub async fn open(
        &self,
        path: impl AsRef<Path>,
        open_options: &OpenOptions,
    ) -> Result<File, Error> {
        let path = path.as_ref();

        tracing::debug!(%path, ?open_options, "opening file");

        let create = open_options
            .inner
            .intersects(OpenOptionsInner::CREATE | OpenOptionsInner::CREATE_NEW);
        let inode_transaction_mode = if create {
            idb::TransactionMode::ReadWrite
        }
        else {
            idb::TransactionMode::ReadOnly
        };

        let transaction = self
            .database
            .transaction(Scope::INODES, inode_transaction_mode)?;

        let mut inode_dirty = false;
        let mut was_created = false;
        let inode = {
            let current_directory = self.state.current_directory.read().await;

            match resolve_inode(
                &transaction,
                &self.state.root_directory,
                &current_directory,
                path,
            )
            .await
            {
                Ok(inode) => {
                    tracing::trace!(inode_id = ?inode.id, "resolved path");

                    if open_options.inner.contains(OpenOptionsInner::CREATE_NEW) {
                        return Err(Error::AlreadyExists {
                            path: path.to_owned(),
                        });
                    }

                    match inode.kind {
                        InodeKind::File { blob_id: _ } => inode.into_owned(),
                        InodeKind::Directory => {
                            return Err(Error::IsADirectory {
                                path: path.to_owned(),
                            });
                        }
                    }
                }
                Err(ResolveInodeError::Database(database_error)) => {
                    return Err(Error::Database(database_error));
                }
                Err(ResolveInodeError::NotADirectory { components, .. }) => {
                    return Err(Error::NotADirectory {
                        path: components.consumed_path().to_owned(),
                    });
                }
                Err(ResolveInodeError::FileNotFound {
                    components,
                    component,
                    current_inode,
                }) => {
                    // check if it the file was not found because the last component is missing and
                    // is the file we want to create
                    let mut can_create_file_name = None;
                    if create {
                        if components.remaining_path().is_empty() {
                            match component {
                                Component::Normal(file_name) => {
                                    can_create_file_name = Some(file_name)
                                }
                                _ => {}
                            }
                        }
                    }

                    if let Some(file_name) = can_create_file_name {
                        let inode_id = transaction
                            .insert_inode(&InsertInode {
                                id: None,
                                parent: Some(current_inode.id),
                                file_name,
                                meta_data: &Metadata::default(),
                                kind: &InodeKind::File { blob_id: None },
                            })
                            .await?;

                        let inode = GetInode {
                            id: inode_id,
                            parent: Some(current_inode.id),
                            file_name: file_name.to_owned(),
                            meta_data: Metadata::default(),
                            kind: InodeKind::File { blob_id: None },
                        };
                        inode_dirty = true;
                        was_created = true;

                        inode
                    }
                    else {
                        return Err(Error::FileNotFound {
                            path: path.to_owned(),
                        });
                    }
                }
            }
        };

        transaction.commit()?;

        Ok(File {
            web_fs: self.clone(),
            open_options: open_options.clone(),
            inode,
            inode_dirty,
            was_created,
        })
    }
}

#[derive(Debug)]
pub struct File {
    web_fs: WebFs,
    open_options: OpenOptions,
    inode: GetInode<Metadata>,
    inode_dirty: bool,
    was_created: bool,
}

impl File {
    pub fn web_fs(&self) -> WebFs {
        self.web_fs.clone()
    }

    pub fn meta_data(&self) -> &Metadata {
        &self.inode.meta_data
    }

    pub fn meta_data_mut(&mut self) -> &mut Metadata {
        self.inode_dirty = true;
        &mut self.inode.meta_data
    }

    pub fn was_created(&self) -> bool {
        self.was_created
    }

    pub async fn flush_inode(&mut self) -> Result<(), Error> {
        if !self.inode_dirty {
            return Ok(());
        }

        let transaction = self
            .web_fs
            .database
            .transaction(Scope::INODES, idb::TransactionMode::ReadWrite)?;

        if self.inode_dirty {
            transaction
                .insert_inode(&InsertInode {
                    id: Some(self.inode.id),
                    parent: self.inode.parent,
                    file_name: &self.inode.file_name,
                    meta_data: &self.inode.meta_data,
                    kind: &self.inode.kind,
                })
                .await?;
        }

        transaction.commit()?;
        self.inode_dirty = false;

        Ok(())
    }

    pub async fn read_blob(&mut self) -> Result<Blob, Error> {
        let transaction = self
            .web_fs
            .database
            .transaction(Scope::BLOBS, idb::TransactionMode::ReadOnly)?;
        let mut blob = None;

        match &mut self.inode.kind {
            InodeKind::File {
                blob_id: blob_id_option,
            } => {
                if let Some(blob_id) = blob_id_option {
                    if let Some(get_blob) = transaction.get_blob(*blob_id).await? {
                        blob = Some(get_blob.data.into());
                    }
                    else {
                        tracing::warn!(inode_id = ?self.inode.id, blob_id = ?blob_id, "blob missing for inode");
                        *blob_id_option = None;
                        self.inode_dirty = true;
                    }
                }
            }
            _ => panic!("inode is not a file"),
        }
        let blob = blob.unwrap_or_else(|| Blob::new(&b""[..]));
        Ok(blob)
    }

    pub async fn read_into(&mut self, buf: &mut BytesMut) -> Result<(), Error> {
        let blob = self.read_blob().await?;

        if let Ok(size) = blob.size().try_into() {
            buf.reserve(size);
        }
        let blob: &web_sys::Blob = blob.as_ref();
        let mut reader = ReadableStream::from_raw(blob.stream())
            .into_async_read()
            .compat();

        while tokio_util::io::read_buf(&mut reader, buf).await? > 0 {}

        Ok(())
    }

    pub async fn read(&mut self) -> Result<Bytes, Error> {
        let mut buf = BytesMut::new();
        self.read_into(&mut buf).await?;
        Ok(buf.freeze())
    }

    pub async fn write_blob(&mut self, blob: Blob) -> Result<(), Error> {
        match &mut self.inode.kind {
            InodeKind::File {
                blob_id: blob_id_option,
            } => {
                let transaction = self
                    .web_fs
                    .database
                    .transaction(Scope::BLOBS, idb::TransactionMode::ReadWrite)?;
                let returned_blob_id = transaction
                    .insert_blob(&InsertBlob {
                        id: *blob_id_option,
                        data: blob.into(),
                    })
                    .await?;

                if blob_id_option.is_none() {
                    *blob_id_option = Some(returned_blob_id);
                    self.inode_dirty = true;
                }
            }
            _ => panic!("inode is not a file"),
        }

        self.flush_inode().await?;
        Ok(())
    }

    pub async fn write(&mut self, data: impl AsRef<[u8]>) -> Result<(), Error> {
        let blob = Blob::new(data.as_ref());
        self.write_blob(blob).await?;
        Ok(())
    }

    pub async fn lock_read(&self) -> FileLockReadGuard {
        self.web_fs.locks.read(self.inode.id).await
    }

    pub async fn lock_write(&self) -> FileLockWriteGuard {
        self.web_fs.locks.write(self.inode.id).await
    }
}

#[derive(Clone)]
pub struct OpenOptions {
    inner: OpenOptionsInner,
}

impl OpenOptions {
    pub fn new() -> Self {
        Self {
            inner: OpenOptionsInner::empty(),
        }
    }

    pub fn create(&mut self, create: bool) -> &mut Self {
        if create {
            self.inner.insert(OpenOptionsInner::CREATE);
        }
        else {
            self.inner.remove(OpenOptionsInner::CREATE_NEW);
        }
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        if create_new {
            self.inner.insert(OpenOptionsInner::CREATE_NEW);
        }
        else {
            self.inner.remove(OpenOptionsInner::CREATE_NEW);
        }
        self
    }
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for OpenOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenOptions")
            .field("create", &self.inner.contains(OpenOptionsInner::CREATE))
            .field(
                "create_new",
                &self.inner.contains(OpenOptionsInner::CREATE_NEW),
            )
            .finish()
    }
}

bitflags! {
    #[derive(Copy, Clone)]
    struct OpenOptionsInner: u8 {
        const CREATE     = 0b00010000;
        const CREATE_NEW = 0b00100000;
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Metadata {
    meta_data: HashMap<String, serde_json::Value>,
}

impl Metadata {
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        if let Some(value) = self.meta_data.get(key) {
            let value = T::deserialize(value)?;
            Ok(Some(value))
        }
        else {
            Ok(None)
        }
    }

    pub fn insert<T: Serialize>(&mut self, key: impl Into<String>, value: &T) -> Result<(), Error> {
        let value = serde_json::to_value(value)?;
        self.meta_data.insert(key.into(), value);
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("web fs error")]
pub enum Error {
    Database(#[from] database::Error),
    Io(#[from] std::io::Error),
    Json(#[from] serde_json::Error),
    #[error("file not found: {path}")]
    FileNotFound {
        path: PathBuf,
    },
    #[error("is a directory: {path}")]
    IsADirectory {
        path: PathBuf,
    },
    #[error("not a directory: {path}")]
    NotADirectory {
        path: PathBuf,
    },
    #[error("already exists: {path}")]
    AlreadyExists {
        path: PathBuf,
    },
}

async fn resolve_inode<'t, 'i, 'p>(
    transaction: &Transaction<'t>,
    root: &'i GetInode<Metadata>,
    current_directory: &'i GetInode<Metadata>,
    path: &'p Path,
) -> Result<Cow<'i, GetInode<Metadata>>, ResolveInodeError<'i, 'p>> {
    let mut current_inode = Cow::Borrowed(current_directory);
    let mut components = path.components();

    while let Some(component) = components.next() {
        match &current_inode.kind {
            InodeKind::File { .. } => {
                return Err(ResolveInodeError::NotADirectory {
                    current_inode,
                    component,
                    components,
                });
            }
            _ => {}
        }

        match component {
            Component::RootDir => {
                current_inode = Cow::Borrowed(root);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                current_inode = if let Some(parent_inode_id) = current_inode.parent {
                    if let Some(parent_inode) = transaction.get_inode(parent_inode_id).await? {
                        Cow::Owned(parent_inode)
                    }
                    else {
                        return Err(ResolveInodeError::FileNotFound {
                            current_inode,
                            component,
                            components,
                        });
                    }
                }
                else {
                    Cow::Borrowed(root)
                };
            }
            Component::Normal(file_name) => {
                current_inode = if let Some(child_inode) = transaction
                    .get_inode_by_name(file_name, Some(current_inode.id))
                    .await?
                {
                    Cow::Owned(child_inode)
                }
                else {
                    return Err(ResolveInodeError::FileNotFound {
                        current_inode,
                        component,
                        components,
                    });
                };
            }
        }
    }

    Ok(current_inode)
}

#[derive(Debug, thiserror::Error)]
#[error("resolve inode error")]
enum ResolveInodeError<'i, 'p> {
    Database(#[from] database::Error),
    NotADirectory {
        current_inode: Cow<'i, GetInode<Metadata>>,
        component: Component<'p>,
        components: Components<'p>,
    },
    FileNotFound {
        current_inode: Cow<'i, GetInode<Metadata>>,
        component: Component<'p>,
        components: Components<'p>,
    },
}
