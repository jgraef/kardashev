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
use futures::AsyncReadExt;
use gloo_file::{
    Blob,
    ObjectUrl,
};
use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};
use tokio::sync::RwLock;
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

pub async fn open(name: Option<&str>) -> Result<WebFs, Error> {
    use tokio::sync::Mutex;

    const DATABASE_NAME: &'static str = "web_fs";
    const DEFAULT_ROOT: &'static str = "default";

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
            database: Database::open(DATABASE_NAME).await?,
            locks: FileLocks::new(),
        };
        *singleton_guard = Some(singleton.clone());
        singleton
    };

    let name = name.unwrap_or(DEFAULT_ROOT);
    WebFs::open_named_root(singleton.database, singleton.locks, name).await
}

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
    async fn open_named_root(
        database: Database,
        locks: FileLocks,
        root: &str,
    ) -> Result<Self, Error> {
        let transaction = database.transaction(Scope::INODES, idb::TransactionMode::ReadOnly)?;
        let root = if let Some(root) = transaction.get_inode_by_name(root, None).await? {
            root
        }
        else {
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
            GetInode {
                id: inode_id,
                parent: None,
                file_name: root.to_owned(),
                meta_data,
                kind: InodeKind::Directory,
            }
        };

        transaction.commit()?;

        Ok(Self::new(database, locks, root))
    }

    fn new(database: Database, locks: FileLocks, root_directory: GetInode<Metadata>) -> Self {
        Self {
            database,
            locks,
            state: Arc::new(State {
                root_directory: root_directory.clone(),
                current_directory: RwLock::new(root_directory),
            }),
        }
    }

    pub async fn open(
        &self,
        path: impl AsRef<Path>,
        open_options: OpenOptions,
    ) -> Result<File, Error> {
        let path = path.as_ref();

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

        let (mut inode, inode_dirty) = {
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
                    if open_options.inner.contains(OpenOptionsInner::CREATE_NEW) {
                        return Err(Error::AlreadyExists {
                            path: path.to_owned(),
                        });
                    }

                    match inode.kind {
                        InodeKind::File { blob_id: _ } => (inode.into_owned(), false),
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
                    mut components,
                    current_inode,
                }) => {
                    // check if it the file was not found because the last component is missing and
                    // is the file we want to create
                    let mut can_create_file_name = None;
                    if create {
                        let next_component = components
                            .next()
                            .expect("expected at least one more path component");
                        if components.next().is_none() {
                            match next_component {
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

                        (inode, true)
                    }
                    else {
                        return Err(Error::FileNotFound {
                            path: path.to_owned(),
                        });
                    }
                }
            }
        };

        let mut data = None;
        if open_options.inner.contains(OpenOptionsInner::READ)
            || (open_options
                .inner
                .intersects(OpenOptionsInner::WRITE | OpenOptionsInner::APPEND)
                && !open_options.inner.contains(OpenOptionsInner::TRUNCATE))
        {
            match &inode.kind {
                InodeKind::File { blob_id } => {
                    if let Some(blob_id) = blob_id {
                        if let Some(blob) = transaction.get_blob(*blob_id).await? {
                            data = Some(blob.data.into());
                        }
                    }
                }
                _ => panic!("inode is not a file"),
            }
        }

        let mut data_dirty = false;
        if open_options.inner.contains(OpenOptionsInner::TRUNCATE) {
            match &mut inode.kind {
                InodeKind::File { blob_id } => {
                    if let Some(_blob_id) = blob_id {
                        assert!(data.is_none());
                        data_dirty = true;
                    }
                    *blob_id = None;
                }
                _ => panic!("inode is not a file"),
            }
        }

        transaction.commit()?;

        Ok(File {
            web_fs: self.clone(),
            open_options,
            inode,
            data: data.unwrap_or_else(|| Blob::new(&b""[..])),
            inode_dirty,
            data_dirty,
        })
    }
}

#[derive(Debug)]
pub struct File {
    web_fs: WebFs,
    open_options: OpenOptions,
    inode: GetInode<Metadata>,
    data: Blob,
    inode_dirty: bool,
    data_dirty: bool,
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

    pub async fn flush(&mut self) -> Result<(), Error> {
        if !self.data_dirty && !self.inode_dirty {
            return Ok(());
        }

        let mut scope = Scope::empty();
        let blob_id = match &self.inode.kind {
            InodeKind::File { blob_id } => *blob_id,
            _ => panic!("inode is not a file"),
        };

        if self.data_dirty && blob_id.is_none() {
            self.inode_dirty = true;
        }
        if self.data_dirty {
            scope |= Scope::BLOBS;
        }
        if self.inode_dirty {
            scope |= Scope::INODES;
        }

        let transaction = self
            .web_fs
            .database
            .transaction(scope, idb::TransactionMode::ReadWrite)?;

        if self.data_dirty {
            let returned_blob_id = transaction
                .insert_blob(&InsertBlob {
                    id: blob_id,
                    data: self.data.clone().into(),
                })
                .await?;

            assert!(blob_id.is_none() || blob_id.unwrap() == returned_blob_id);

            match &mut self.inode.kind {
                InodeKind::File { blob_id } => {
                    *blob_id = Some(returned_blob_id);
                }
                _ => panic!("inode is not a file"),
            }
        }

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
        self.data_dirty = false;

        Ok(())
    }

    pub fn blob(&self) -> Blob {
        self.data.clone()
    }

    pub fn object_url(&self) -> ObjectUrl {
        self.data.clone().into()
    }

    pub async fn read_into(&self, buf: &mut Vec<u8>) -> Result<(), Error> {
        if let Ok(size) = self.data.size().try_into() {
            buf.reserve(size);
        }
        let blob: &web_sys::Blob = self.data.as_ref();
        let mut reader = ReadableStream::from_raw(blob.stream()).into_async_read();
        reader.read_to_end(buf).await?;
        Ok(())
    }

    pub async fn read(&self) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::with_capacity(self.data.size().try_into().unwrap_or_default());
        self.read_into(&mut buf).await?;
        Ok(buf)
    }

    pub async fn write(&mut self, data: impl AsRef<[u8]>) -> Result<(), Error> {
        self.data = Blob::new(data.as_ref());
        self.data_dirty = true;
        self.flush().await?;
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

    pub fn read(&mut self, read: bool) -> &mut Self {
        if read {
            self.inner.insert(OpenOptionsInner::READ);
        }
        else {
            self.inner.remove(OpenOptionsInner::READ);
        }
        self
    }

    pub fn write(&mut self, write: bool) -> &mut Self {
        if write {
            self.inner.insert(OpenOptionsInner::WRITE);
        }
        else {
            self.inner.remove(OpenOptionsInner::WRITE);
        }
        self
    }

    pub fn append(&mut self, append: bool) -> &mut Self {
        if append {
            self.inner.insert(OpenOptionsInner::APPEND);
        }
        else {
            self.inner.remove(OpenOptionsInner::APPEND);
        }
        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        if truncate {
            self.inner.insert(OpenOptionsInner::TRUNCATE);
        }
        else {
            self.inner.remove(OpenOptionsInner::TRUNCATE);
        }
        self
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
        Self {
            inner: OpenOptionsInner::READ,
        }
    }
}

impl Debug for OpenOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenOptions")
            .field("read", &self.inner.contains(OpenOptionsInner::READ))
            .field("write", &self.inner.contains(OpenOptionsInner::WRITE))
            .field("append", &self.inner.contains(OpenOptionsInner::APPEND))
            .field("truncate", &self.inner.contains(OpenOptionsInner::TRUNCATE))
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
        const READ       = 0b00000001;
        const WRITE      = 0b00000010;
        const APPEND     = 0b00000100;
        const TRUNCATE   = 0b00001000;
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

#[derive(Debug, thiserror::Error)]
#[error("web_fs error")]
pub enum Error {
    Database(#[from] database::Error),
    Io(#[from] std::io::Error),
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
                            components,
                        });
                    }
                }
                else {
                    Cow::Borrowed(root)
                };
            }
            Component::Normal(component) => {
                current_inode = if let Some(child_inode) = transaction
                    .get_inode_by_name(component, current_inode.parent)
                    .await?
                {
                    Cow::Owned(child_inode)
                }
                else {
                    return Err(ResolveInodeError::FileNotFound {
                        current_inode,
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
        components: Components<'p>,
    },
    FileNotFound {
        current_inode: Cow<'i, GetInode<Metadata>>,
        components: Components<'p>,
    },
}
