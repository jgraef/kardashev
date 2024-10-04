pub mod image_load;

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
    Events,
};
use kardashev_protocol::assets::{
    self as dist,
    AssetId,
};
use tokio::sync::mpsc;
use url::Url;

use crate::{
    utils::spawn_local_and_handle_error,
    world::{
        Plugin,
        RegisterPluginContext,
        RunSystemContext,
        System,
    },
};

#[derive(Debug, thiserror::Error)]
#[error("asset loader error")]
pub enum Error {
    Reqwest(#[from] reqwest::Error),
    AssetNotFound(#[from] AssetNotFound),
    ImageLoad(#[from] image_load::LoadImageError),
    Graphics(#[from] crate::graphics::Error),
    Client(#[from] kardashev_client::Error),
    AssetParse(#[from] kardashev_protocol::assets::AssetParseError),
}

#[derive(Debug, thiserror::Error)]
#[error("asset not found: {asset_id}")]
pub struct AssetNotFound {
    pub asset_id: AssetId,
}

/// Handle to the asset server that loads assets.
#[derive(Clone, Debug)]
pub struct AssetServer {
    tx_command: mpsc::UnboundedSender<Command>,
}

impl AssetServer {
    fn send_command(&self, command: Command) {
        self.tx_command.send(command).expect("asset server died");
    }

    /// Loads an asset of type `A` with the given `asset_id`.
    pub fn load<A: Asset>(&self, asset_id: AssetId) -> Load<A> {
        let (tx, rx) = oneshot::channel();

        self.send_command(Command::Load {
            load_request: DynAssetLoadRequest::new(asset_id, tx),
        });

        Load {
            asset_id,
            state: LoadState::Wait { rx },
        }
    }

    pub fn register_asset_type<A: Asset>(&self) {
        self.send_command(Command::RegisterAssetType {
            asset_type: DynAssetType::new::<A>(),
        })
    }
}

/// Trait for assets that can be loaded from the asset API.
///
/// See also [`GpuAsset`][`crate::rendering::loading::GpuAsset`].
pub trait Asset: Sized + Send + Sync + 'static {
    type Dist;
    type LoadError: std::error::Error + Send + Sync;

    fn load<'a, 'b: 'a>(
        asset_id: AssetId,
        loader: &'a mut Loader<'b>,
    ) -> impl Future<Output = Result<Self, Self::LoadError>> + 'a;
}

/// An asset in the process of being loaded.
///
/// This is a future and can be polled for the loaded asset. It is also a
/// component, and can be attached to entities. The [`AssetLoaderSystem`] will
/// then check if the load is complete, remove the [`Load`] and attach the
/// loaded asset to the entity.
///
/// # TODO
///
/// Document panic behavior
#[derive(Debug)]
pub struct Load<A: Asset> {
    asset_id: AssetId,
    state: LoadState<A>,
}

impl<A: Asset> Load<A> {
    pub fn new(asset_id: AssetId) -> Self {
        Self {
            asset_id,
            state: LoadState::New,
        }
    }

    pub fn asset_id(&self) -> AssetId {
        self.asset_id
    }

    pub fn try_get(&mut self) -> Option<Result<A, <A as Asset>::LoadError>> {
        match &mut self.state {
            LoadState::New => None,
            LoadState::Wait { rx } => rx.try_recv().expect("asset load request sender dropped"),
            LoadState::Done => panic!("load request result was already taken out"),
        }
    }
}

impl<A: Asset> Future for Load<A> {
    type Output = Result<A, <A as Asset>::LoadError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.state {
            LoadState::New => panic!("load request wasn't started yet"),
            LoadState::Wait { rx } => {
                rx.poll_unpin(cx).map(|result| {
                    self.state = LoadState::Done;
                    result.expect("asset load request sender dropped")
                })
            }
            LoadState::Done => panic!("load request result was already taken out"),
        }
    }
}

#[derive(Debug)]
enum LoadState<A: Asset> {
    New,
    Wait {
        rx: oneshot::Receiver<Result<A, <A as Asset>::LoadError>>,
    },
    Done,
}

#[derive(Debug)]
struct Reactor {
    client: AssetClient,
    assets: dist::Assets,
    cache: Cache,
    rx_command: mpsc::UnboundedReceiver<Command>,
}

impl Reactor {
    fn spawn(client: AssetClient, rx_command: mpsc::UnboundedReceiver<Command>) {
        spawn_local_and_handle_error(async move {
            let manifest = client.get_manifest().await?;

            let mut dist_asset_types = dist::AssetTypes::default();
            dist_asset_types.with_builtin();
            let assets = manifest.assets.parse(&dist_asset_types)?;
            for ty in assets.unrecognized_types() {
                tracing::warn!("unrecognized asset type: {ty:?}");
            }

            let reactor = Self {
                client,
                assets,
                cache: Cache::default(),
                rx_command,
            };

            reactor.run().await
        });
    }

    async fn run(mut self) -> Result<(), Error> {
        let mut events = self
            .client
            .events()
            .await
            .map_err(|error| tracing::error!(?error, "asset client events doesn't work"))
            .ok();
        async fn next_event(
            events: &mut Option<Events>,
        ) -> Result<dist::Event, kardashev_client::Error> {
            if let Some(events) = events {
                events.next().await
            }
            else {
                std::future::pending().await
            }
        }

        loop {
            tokio::select! {
                command_opt = self.rx_command.recv() => {
                    let Some(command) = command_opt else { break; };
                    self.handle_command(command).await?;
                }
                event_result = next_event(&mut events) => {
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
            Command::RegisterAssetType { asset_type: _ } => {
                // todo
            }
        }

        Ok(())
    }

    async fn handle_event(&mut self, event: dist::Event) -> Result<(), Error> {
        match event {
            dist::Event::Changed { asset_ids } => {
                tracing::debug!(?asset_ids, "assets changed");
                // todo: the specified asset was changed and can be reloaded
            }
            dist::Event::Lagged => {}
        }

        Ok(())
    }
}

#[derive(Debug)]
enum Command {
    Load { load_request: DynAssetLoadRequest },
    RegisterAssetType { asset_type: DynAssetType },
}

/// Context for [`Asset::load`]
#[derive(Debug)]
pub struct Loader<'a> {
    pub dist_assets: &'a dist::Assets,
    pub client: &'a AssetClient,
    pub cache: &'a mut Cache,
}

impl<'a> Loader<'a> {
    fn from_reactor(reactor: &'a mut Reactor) -> Self {
        Self {
            dist_assets: &reactor.assets,
            client: &reactor.client,
            cache: &mut reactor.cache,
        }
    }
}

/// In-memory cache for asset data
///
/// # TODO
///
/// Either change this or make a second cache that caches the raw bytes
/// downloaded. This can then be stored in IndexedDB.
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

/// [`System`] that queries [`Load<A>`s](Load), loads them, and attaches the
/// loaded asset.
#[derive(Default)]
pub struct AssetLoaderSystem {
    command_buffer: hecs::CommandBuffer,
}

impl System for AssetLoaderSystem {
    fn label(&self) -> &'static str {
        "asset-loader"
    }

    async fn run<'s: 'c, 'c: 'd, 'd>(
        &'s mut self,
        context: &'d mut RunSystemContext<'c>,
    ) -> Result<(), crate::error::Error> {
        let Some(asset_type_registry) = context.resources.get::<AssetTypeRegistry>()
        else {
            tracing::warn!("missing AssetTypeRegistry resource");
            return Ok(());
        };

        let asset_server = context
            .resources
            .get::<AssetServer>()
            .expect("AssetServer resource missing");

        for asset_type in &asset_type_registry.asset_types {
            tracing::trace!(
                asset_type = asset_type.asset_type_name(),
                "running asset loader system"
            );
            asset_type.loader_system(asset_server, &mut context.world, &mut self.command_buffer);
        }

        // note: we use our own command buffer so that loaded assets are already
        // attached right after this system has run
        self.command_buffer.run_on(&mut context.world);

        Ok(())
    }
}

impl Debug for AssetLoaderSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetLoaderSystem").finish_non_exhaustive()
    }
}

/// Registry for asset types.
///
/// This is a resource that can be used to register asset types.
///
/// This is used by the [`AssetLoaderSystem`] to query the world for registered
/// asset types, and by the asset server to parse metadata from the asset
/// manifest.
#[derive(Clone, Debug)]
pub struct AssetTypeRegistry {
    asset_types: Vec<DynAssetType>,
    asset_server: AssetServer,
}

impl AssetTypeRegistry {
    fn new(asset_server: AssetServer) -> Self {
        Self {
            asset_types: vec![],
            asset_server,
        }
    }

    pub fn register<A: Asset>(&mut self) -> &mut Self {
        self.asset_types.push(DynAssetType::new::<A>());
        self.asset_server.register_asset_type::<A>();
        self
    }
}

#[derive(Debug)]
pub struct AssetsPlugin {
    client: AssetClient,
}

impl AssetsPlugin {
    pub fn from_url(asset_url: Url) -> Self {
        Self::from_client(AssetClient::new(asset_url))
    }

    pub fn from_client(client: AssetClient) -> Self {
        Self { client }
    }
}

impl Plugin for AssetsPlugin {
    fn register(self, context: RegisterPluginContext) {
        let (tx_command, rx_command) = mpsc::unbounded_channel();
        Reactor::spawn(self.client.clone(), rx_command);
        let asset_server = AssetServer { tx_command };

        context.resources.insert(asset_server.clone());
        context
            .resources
            .insert(AssetTypeRegistry::new(asset_server));
        context
            .scheduler
            .add_update_system(AssetLoaderSystem::default());
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
    fn loader_system<'w>(
        &self,
        asset_server: &AssetServer,
        world: &'w mut hecs::World,
        command_buffer: &'w mut hecs::CommandBuffer,
    );
}

struct DynAssetTypeImpl<A> {
    _ty: PhantomData<A>,
}

impl<A: Asset> DynAssetTypeTrait for DynAssetTypeImpl<A> {
    fn asset_type_name(&self) -> &'static str {
        type_name::<A>()
    }

    fn loader_system<'w>(
        &self,
        asset_server: &AssetServer,
        world: &'w mut hecs::World,
        command_buffer: &'w mut hecs::CommandBuffer,
    ) {
        let query = world.query_mut::<hecs::Without<&mut Load<A>, &A>>();

        for (entity, load) in query {
            match &mut load.state {
                LoadState::New => {
                    let (tx, rx) = oneshot::channel();
                    asset_server.send_command(Command::Load {
                        load_request: DynAssetLoadRequest::new(load.asset_id, tx),
                    });
                    load.state = LoadState::Wait { rx };
                }
                LoadState::Wait { rx } => {
                    if let Some(result) = rx.try_recv().expect("asset load request sender dropped")
                    {
                        match result {
                            Ok(asset) => {
                                tracing::debug!(asset_id = %load.asset_id, "asset loaded");
                                command_buffer.insert_one(entity, asset);
                            }
                            Err(error) => {
                                tracing::error!(asset_id = %load.asset_id, ?error, "failed to load asset");
                            }
                        }

                        command_buffer.remove_one::<Load<A>>(entity);
                        load.state = LoadState::Done;
                    }
                }
                LoadState::Done => panic!("load request result was already taken out"),
            }
        }
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
            let _ = self.tx.send(result);
        })
    }
}
