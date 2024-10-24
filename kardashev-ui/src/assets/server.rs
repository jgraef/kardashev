use kardashev_client::{
    AssetClient,
    Events,
};
use kardashev_protocol::assets::{
    self as dist,
    AssetId,
};
use tokio::sync::{
    mpsc,
    oneshot,
};

use crate::{
    assets::{
        dyn_type::{
            DynAssetLoadRequest,
            DynAssetType,
        },
        load::{
            LoadAssetContext,
            LoadAsync,
            LoadFromAsset,
        },
        store::AssetStore,
        Error,
    },
    utils::{
        any_cache::AnyArcCache,
        futures::spawn_local_and_handle_error,
    },
};

/// Handle to the asset server that loads assets.
#[derive(Clone, Debug)]
pub struct AssetServer {
    tx_command: mpsc::UnboundedSender<Command>,
}

impl AssetServer {
    pub fn new(client: AssetClient) -> Self {
        let (tx_command, rx_command) = mpsc::unbounded_channel();
        Reactor::spawn(client, rx_command);
        AssetServer { tx_command }
    }

    pub(super) fn send_command(&self, command: Command) {
        self.tx_command.send(command).expect("asset server died");
    }

    pub(super) fn start_load<A: LoadFromAsset>(
        &self,
        asset_id: AssetId,
        args: <A as LoadFromAsset>::Args,
    ) -> oneshot::Receiver<Result<A, <A as LoadFromAsset>::Error>> {
        let (tx, rx) = oneshot::channel();
        self.send_command(Command::Load {
            load_request: DynAssetLoadRequest::new(asset_id, args, tx),
        });
        rx
    }

    #[allow(dead_code)]
    pub fn load<A: LoadFromAsset>(
        &self,
        asset_id: AssetId,
        args: <A as LoadFromAsset>::Args,
    ) -> LoadAsync<A> {
        LoadAsync {
            rx: self.start_load(asset_id, args),
        }
    }

    pub fn register_asset_type<A: LoadFromAsset>(&self) {
        self.send_command(Command::RegisterAssetType {
            asset_type: DynAssetType::new::<A>(),
        })
    }
}

#[derive(Debug)]
struct Reactor {
    client: AssetClient,
    asset_store: AssetStore,
    assets: dist::Assets,
    cache: AnyArcCache<AssetId>,
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

            let asset_store = AssetStore::new().await?;

            let reactor = Self {
                client,
                asset_store,
                assets,
                cache: AnyArcCache::default(),
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
                let asset_store = self.asset_store.lock().await;
                let mut loader = LoadAssetContext {
                    dist_assets: &self.assets,
                    client: &self.client,
                    asset_store: &asset_store,
                    cache: &mut self.cache,
                };
                load_request.load(&mut loader).await;
            }
            Command::RegisterAssetType { asset_type } => {
                let _ = asset_type;
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
pub(super) enum Command {
    Load { load_request: DynAssetLoadRequest },
    RegisterAssetType { asset_type: DynAssetType },
}
