use std::{
    any::type_name,
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    ops::Deref,
    pin::Pin,
};

use kardashev_protocol::assets::AssetId;
use tokio::sync::oneshot;

use crate::assets::{
    load::{
        Load,
        LoadAssetContext,
        LoadAssetState,
        LoadFromAsset,
    },
    server::AssetServer,
};

#[derive(Clone, Copy)]
pub(super) struct DynAssetType {
    inner: &'static dyn DynAssetTypeTrait,
}

impl DynAssetType {
    pub const fn new<A: LoadFromAsset>() -> Self {
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

pub(super) trait DynAssetTypeTrait {
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

impl<A: LoadFromAsset> DynAssetTypeTrait for DynAssetTypeImpl<A> {
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
                LoadAssetState::New { args } => {
                    let args = args
                        .take()
                        .expect("LoadAssetState::new without args (invalid state)");
                    let rx = asset_server.start_load(load.asset_id, args);
                    load.state = LoadAssetState::Wait { rx };
                }
                LoadAssetState::Wait { rx } => {
                    match rx.try_recv() {
                        Ok(result) => {
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
                            load.state = LoadAssetState::Done;
                        }
                        Err(oneshot::error::TryRecvError::Closed) => {
                            panic!("asset load request sender dropped")
                        }
                        Err(oneshot::error::TryRecvError::Empty) => {}
                    }
                }
                LoadAssetState::Done => panic!("load request result was already taken out"),
            }
        }
    }
}

impl<A> Clone for DynAssetTypeImpl<A> {
    fn clone(&self) -> Self {
        Self { _ty: PhantomData }
    }
}

pub(super) struct DynAssetLoadRequest {
    inner: Box<dyn DynAssetLoadRequestTrait>,
}

impl DynAssetLoadRequest {
    pub fn new<A: LoadFromAsset>(
        asset_id: AssetId,
        args: <A as LoadFromAsset>::Args,
        tx: oneshot::Sender<Result<A, <A as LoadFromAsset>::Error>>,
    ) -> Self {
        Self {
            inner: Box::new(DynAssetLoadRequestImpl { asset_id, args, tx }),
        }
    }

    pub fn asset_type_name(&self) -> &'static str {
        self.inner.asset_type_name()
    }

    pub fn asset_id(&self) -> AssetId {
        self.inner.asset_id()
    }

    pub async fn load<'a>(self, context: &'a mut LoadAssetContext<'a>) {
        self.inner.load(context).await;
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
        context: &'a mut LoadAssetContext<'a>,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
}

struct DynAssetLoadRequestImpl<A: LoadFromAsset> {
    asset_id: AssetId,
    args: <A as LoadFromAsset>::Args,
    tx: oneshot::Sender<Result<A, <A as LoadFromAsset>::Error>>,
}

impl<A: LoadFromAsset> DynAssetLoadRequestTrait for DynAssetLoadRequestImpl<A> {
    fn asset_type_name(&self) -> &'static str {
        type_name::<A>()
    }

    fn asset_id(&self) -> AssetId {
        self.asset_id
    }

    fn load<'a>(
        self: Box<Self>,
        context: &'a mut LoadAssetContext<'a>,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(async move {
            let result = A::load(self.asset_id, self.args, context).await;
            if let Err(error) = &result {
                tracing::error!(?error, "asset load failed");
            }
            let _ = self.tx.send(result);
        })
    }
}
