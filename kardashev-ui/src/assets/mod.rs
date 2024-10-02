mod image_load;

use std::{
    any::{
        type_name,
        Any,
        TypeId,
    },
    collections::{
        hash_map,
        HashMap,
    },
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    ops::Deref,
    pin::Pin,
    sync::Arc,
    task::{
        Context,
        Poll,
    },
};

use futures::{
    channel::oneshot,
    FutureExt,
};
use kardashev_client::{
    AssetClient,
    DownloadError,
    DownloadFile,
};
use kardashev_protocol::assets::{
    self as dist,
    AssetId,
};
use tokio::sync::mpsc;

pub use self::image_load::{
    load_image,
    LoadImage,
    LoadImageError,
};
use crate::utils::spawn_local_and_handle_error;

#[derive(Debug, thiserror::Error)]
#[error("asset loader error")]
pub enum Error {
    Reqwest(#[from] reqwest::Error),
    AssetNotFound(#[from] AssetNotFound),
    ImageLoad(#[from] image_load::LoadImageError),
    Graphics(#[from] crate::graphics::Error),
    Client(#[from] kardashev_client::Error),
}

#[derive(Debug, thiserror::Error)]
#[error("asset not found: {asset_id}")]
pub struct AssetNotFound {
    asset_id: AssetId,
}

#[derive(Debug, Default)]
pub struct AssetServerBuilder {
    registered_asset_types: Vec<DynAssetType>,
    client: Option<AssetClient>,
}

impl AssetServerBuilder {
    pub fn register_asset<A: Asset>(mut self) -> Self {
        self.registered_asset_types.push(DynAssetType::new::<A>());
        self
    }

    pub fn with_client(mut self, client: AssetClient) -> Self {
        self.client = Some(client);
        self
    }

    pub fn build(self) -> AssetServer {
        let (tx_command, rx_command) = mpsc::unbounded_channel();
        let client = self.client.expect("missing AssetClient");
        Reactor::spawn(client, rx_command, self.registered_asset_types);
        AssetServer { tx_command }
    }
}

#[derive(Clone, Debug)]
pub struct AssetServer {
    tx_command: mpsc::UnboundedSender<Command>,
}

impl AssetServer {
    pub fn builder() -> AssetServerBuilder {
        AssetServerBuilder::default()
    }

    fn send_command(&self, command: Command) {
        self.tx_command.send(command).expect("asset server died");
    }

    pub fn load<A: Asset>(&self, asset_id: AssetId) -> Load<A> {
        let (tx, rx) = oneshot::channel();

        self.send_command(Command::Load {
            load_request: DynAssetLoadRequest::new(asset_id, tx),
        });

        Load {
            asset_id,
            rx: Some(rx),
        }
    }
}

pub trait Asset: Sized + 'static {
    type Dist;
    type LoadError: std::error::Error;

    fn parse_dist_manifest(manifest: &dist::Manifest, refs: &mut HashMap<AssetId, usize>);
    fn get_from_dist_manifest(manifest: &dist::Manifest, index: usize) -> Option<&Self::Dist>;
    fn load<'a>(
        asset_id: AssetId,
        loader: &'a mut Loader<'a>,
    ) -> impl Future<Output = Result<Self, Self::LoadError>> + 'a;
}

#[derive(Debug)]
pub struct Load<A: Asset> {
    asset_id: AssetId,
    rx: Option<oneshot::Receiver<Result<A, <A as Asset>::LoadError>>>,
}

impl<A: Asset> Load<A> {
    pub fn asset_id(&self) -> AssetId {
        self.asset_id
    }

    pub fn try_get(&mut self) -> Option<Result<A, <A as Asset>::LoadError>> {
        self.rx
            .as_mut()
            .expect("load request result was already taken out")
            .try_recv()
            .expect("asset load request sender dropped")
    }
}

impl<A: Asset> Future for Load<A> {
    type Output = Result<A, <A as Asset>::LoadError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.rx
            .as_mut()
            .expect("future already returned result")
            .poll_unpin(cx)
            .map(|result| result.expect("asset load request sender dropped"))
    }
}

#[derive(Debug)]
struct Reactor {
    client: AssetClient,
    metadata: Metadata,
    cache: Cache,
    rx_command: mpsc::UnboundedReceiver<Command>,
}

impl Reactor {
    fn spawn(
        client: AssetClient,
        rx_command: mpsc::UnboundedReceiver<Command>,
        registered_asset_types: Vec<DynAssetType>,
    ) {
        spawn_local_and_handle_error(async move {
            let manifest = client.get_manifest().await?;
            let metadata = Metadata::from_manifest(manifest, &registered_asset_types);

            let reactor = Self {
                client,
                metadata,
                cache: Cache::default(),
                rx_command,
            };

            reactor.run().await
        });
    }

    async fn run(mut self) -> Result<(), Error> {
        let mut events = self.client.events().await?;

        loop {
            tokio::select! {
                command_opt = self.rx_command.recv() => {
                    let Some(command) = command_opt else { break; };
                    self.handle_command(command).await?;
                }
                event_result = events.next() => {
                    self.handle_event(event_result?).await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_command(&mut self, command: Command) -> Result<(), Error> {
        match command {
            Command::Load { load_request } => {
                tracing::debug!(asset_id = %load_request.asset_id(), asset_type = load_request.asset_type_name(), "loading asset");
                let mut loader = Loader::from_reactor(self);
                load_request.load(&mut loader).await;
            }
        }

        Ok(())
    }

    async fn handle_event(&mut self, event: dist::Message) -> Result<(), Error> {
        match event {
            dist::Message::Changed { asset_id } => {
                tracing::debug!(%asset_id, "asset changed");
                // todo: the specified asset was changed and can be reloaded
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum Command {
    Load { load_request: DynAssetLoadRequest },
}

#[derive(Debug)]
pub struct Metadata {
    manifest: dist::Manifest,
    refs: HashMap<AssetId, usize>,
}

impl Metadata {
    fn from_manifest(manifest: dist::Manifest, registered_asset_types: &[DynAssetType]) -> Self {
        let mut refs = HashMap::new();
        for asset_type in registered_asset_types {
            asset_type.parse_dist_manifest(&manifest, &mut refs)
        }
        Self { manifest, refs }
    }

    pub fn get<A: Asset>(&self, asset_id: AssetId) -> Result<&A::Dist, AssetNotFound> {
        let index = self
            .refs
            .get(&asset_id)
            .copied()
            .ok_or_else(|| AssetNotFound { asset_id })?;

        A::get_from_dist_manifest(&self.manifest, index).ok_or_else(|| AssetNotFound { asset_id })
    }
}

#[derive(Debug)]
pub struct Loader<'a> {
    pub metadata: &'a Metadata,
    pub client: &'a AssetClient,
    pub cache: &'a mut Cache,
}

impl<'a> Loader<'a> {
    fn from_reactor(reactor: &'a mut Reactor) -> Self {
        Self {
            metadata: &reactor.metadata,
            client: &reactor.client,
            cache: &mut reactor.cache,
        }
    }
}

#[derive(Default)]
pub struct Cache {
    cache: HashMap<(AssetId, TypeId), Arc<dyn Any + Send + Sync + 'static>>,
}

impl Cache {
    pub fn get<T>(&self, asset_id: AssetId) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.cache
            .get(&(asset_id, TypeId::of::<T>()))
            .map(|data| Arc::downcast::<T>(data.clone()).unwrap())
    }

    pub fn insert<T>(&mut self, asset_id: AssetId, data: Arc<T>)
    where
        T: Send + Sync + 'static,
    {
        self.cache.insert((asset_id, TypeId::of::<T>()), data);
    }

    pub async fn get_or_try_insert_async<T, F, Fut, E>(
        &mut self,
        asset_id: AssetId,
        f: F,
    ) -> Result<Arc<T>, E>
    where
        T: Send + Sync + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Arc<T>, E>>,
    {
        match self.cache.entry((asset_id, TypeId::of::<T>())) {
            hash_map::Entry::Occupied(occupied) => {
                Ok(Arc::downcast::<T>(occupied.get().clone()).unwrap())
            }
            hash_map::Entry::Vacant(vacant) => {
                let data = f().await?;
                vacant.insert(data.clone());
                Ok(data)
            }
        }
    }
}

impl Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cache").finish_non_exhaustive()
    }
}

#[derive(Clone, Copy)]
struct DynAssetType {
    inner: &'static dyn DynAssetTypeTrait,
}

impl DynAssetType {
    pub const fn new<A: Asset>() -> Self {
        Self {
            inner: &DynAssetTypeImpl {
                _ty: PhantomData::<A>,
            },
        }
    }
}

impl Deref for DynAssetType {
    type Target = dyn DynAssetTypeTrait;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl Debug for DynAssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DynAssetType::<{}>", self.inner.asset_type_name())
    }
}

trait DynAssetTypeTrait {
    fn asset_type_name(&self) -> &'static str;
    fn parse_dist_manifest(&self, manifest: &dist::Manifest, refs: &mut HashMap<AssetId, usize>);
}

struct DynAssetTypeImpl<A> {
    _ty: PhantomData<A>,
}

impl<A: Asset> DynAssetTypeTrait for DynAssetTypeImpl<A> {
    fn asset_type_name(&self) -> &'static str {
        type_name::<A>()
    }

    fn parse_dist_manifest(&self, manifest: &dist::Manifest, refs: &mut HashMap<AssetId, usize>) {
        A::parse_dist_manifest(manifest, refs);
    }
}

impl<A> Clone for DynAssetTypeImpl<A> {
    fn clone(&self) -> Self {
        Self { _ty: PhantomData }
    }
}

struct DynAssetLoadRequest {
    inner: Box<dyn DynAssetLoadRequestTrait>,
}

impl DynAssetLoadRequest {
    pub fn new<A: Asset>(
        asset_id: AssetId,
        tx: oneshot::Sender<Result<A, <A as Asset>::LoadError>>,
    ) -> Self {
        Self {
            inner: Box::new(DynAssetLoadRequestImpl { asset_id, tx }),
        }
    }

    pub fn asset_type_name(&self) -> &'static str {
        self.inner.asset_type_name()
    }

    pub fn asset_id(&self) -> AssetId {
        self.inner.asset_id()
    }

    pub async fn load<'a>(self, loader: &'a mut Loader<'a>) {
        self.inner.load(loader).await;
    }
}

impl Debug for DynAssetLoadRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DynAssetLoadRequest::<{}>", self.inner.asset_type_name())
    }
}

trait DynAssetLoadRequestTrait {
    fn asset_type_name(&self) -> &'static str;
    fn asset_id(&self) -> AssetId;

    fn load<'a>(
        self: Box<Self>,
        loader: &'a mut Loader<'a>,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
}

struct DynAssetLoadRequestImpl<A: Asset> {
    asset_id: AssetId,
    tx: oneshot::Sender<Result<A, <A as Asset>::LoadError>>,
}

impl<A: Asset> DynAssetLoadRequestTrait for DynAssetLoadRequestImpl<A> {
    fn asset_type_name(&self) -> &'static str {
        type_name::<A>()
    }

    fn asset_id(&self) -> AssetId {
        self.asset_id
    }

    fn load<'a>(
        self: Box<Self>,
        loader: &'a mut Loader<'a>,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(async move {
            let result = A::load(self.asset_id, loader).await;
            if let Err(error) = &result {
                tracing::error!(?error, "asset load failed");
            }
            self.tx.send(result);
        })
    }
}
