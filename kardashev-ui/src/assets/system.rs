use std::fmt::Debug;

use kardashev_client::AssetClient;
use url::Url;

use crate::{
    assets::{
        dyn_type::DynAssetType,
        load::LoadFromAsset,
        server::AssetServer,
    },
    world::{
        Plugin,
        RegisterPluginContext,
        RunSystemContext,
        System,
    },
};

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

    pub fn register<A: LoadFromAsset>(&mut self) -> &mut Self {
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
        let asset_server = AssetServer::new(self.client.clone());

        context.resources.insert(asset_server.clone());
        context
            .resources
            .insert(AssetTypeRegistry::new(asset_server));
        context
            .scheduler
            .add_update_system(AssetLoaderSystem::default());
    }
}
