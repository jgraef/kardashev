use std::{
    fmt::Debug,
    future::Future,
};

use kardashev_client::AssetClient;
use kardashev_protocol::assets::{
    self as dist,
    AssetId,
    HasAssetId,
};
use tokio::sync::oneshot;

use crate::{
    assets::MaybeHasAssetId,
    utils::any_cache::AnyCache,
};

/// Trait for assets that can be loaded from the asset API.
///
/// See also [`GpuAsset`][`crate::rendering::loading::GpuAsset`].
pub trait LoadFromAsset: MaybeHasAssetId + Sized + Send + Sync + 'static {
    type Dist;
    type Args: Debug + Send + Sync + 'static;
    type Error: std::error::Error + Send + Sync;

    fn load<'a, 'b: 'a>(
        asset_id: AssetId,
        args: Self::Args,
        context: &'a mut LoadAssetContext<'b>,
    ) -> impl Future<Output = Result<Self, Self::Error>> + 'a;
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
pub struct Load<A: LoadFromAsset> {
    pub(super) asset_id: AssetId,
    pub(super) state: LoadAssetState<A>,
}

impl<A: LoadFromAsset> HasAssetId for Load<A> {
    fn asset_id(&self) -> AssetId {
        self.asset_id
    }
}

impl<A: LoadFromAsset> Load<A> {
    pub fn new(asset_id: AssetId) -> Self
    where
        <A as LoadFromAsset>::Args: Default,
    {
        Self::with_args(asset_id, Default::default())
    }

    pub fn with_args(asset_id: AssetId, args: <A as LoadFromAsset>::Args) -> Self {
        Self {
            asset_id,
            state: LoadAssetState::New { args: Some(args) },
        }
    }

    pub fn try_get(&mut self) -> Option<Result<A, <A as LoadFromAsset>::Error>> {
        match &mut self.state {
            LoadAssetState::New { .. } => None,
            LoadAssetState::Wait { rx } => {
                Some(rx.try_recv().expect("asset load request sender dropped"))
            }
            LoadAssetState::Done => panic!("load request result was already taken out"),
        }
    }
}

#[derive(Debug)]
pub(super) enum LoadAssetState<A: LoadFromAsset> {
    New {
        args: Option<<A as LoadFromAsset>::Args>,
    },
    Wait {
        rx: oneshot::Receiver<Result<A, <A as LoadFromAsset>::Error>>,
    },
    Done,
}

/// Context for [`Asset::load`]
#[derive(Debug)]
pub struct LoadAssetContext<'a> {
    pub dist_assets: &'a dist::Assets,
    pub client: &'a AssetClient,
    pub cache: &'a mut AnyCache<AssetId>,
}
