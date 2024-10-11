use std::{
    any::type_name,
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    num::NonZeroUsize,
    sync::{
        atomic::AtomicUsize,
        Arc,
    },
    task::Poll,
};

use kardashev_protocol::assets::AssetId;
use linear_map::LinearMap;

use crate::{
    assets::MaybeHasAssetId,
    graphics::{
        material::Material,
        mesh::Mesh,
        pipeline::{
            Pipeline,
            RenderTarget,
        },
        Backend,
        BackendId,
        Error,
    },
    utils::{
        any_cache::AnyArcCache,
        thread_local_cell::ThreadLocalCell,
    },
    world::system::{
        System,
        SystemContext,
    },
};

/// Trait defining how an [`Asset`] can be loaded to the GPU.
///
/// The associated constant [`Self::Loaded`] should contain the GPU handles.
/// This will be put into a map with on entry per backend (GPU) and then used by
/// the rendering pipeline.
pub trait GpuAsset: MaybeHasAssetId {
    type Loaded: 'static;

    fn load(&self, context: &LoadContext) -> Result<Self::Loaded, Error>;
}

/// Struct holding the handles to on-GPU resources.
///
/// This is usually used as an component (e.g. as a `OnGpu<Mesh>`).
pub struct OnGpu<A: GpuAsset> {
    loaded: LinearMap<BackendId, BackendResource<<A as GpuAsset>::Loaded>>,
}

impl<A: GpuAsset> Default for OnGpu<A> {
    fn default() -> Self {
        Self {
            loaded: LinearMap::new(),
        }
    }
}

impl<A: GpuAsset> Debug for OnGpu<A>
where
    <A as GpuAsset>::Loaded: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnGpu")
            .field("loaded", &self.loaded)
            .finish()
    }
}

impl<A: GpuAsset> OnGpu<A> {
    /// Returns the [`BackendResourceHandle`] for the specified `backend_id`.
    pub fn get(&self, backend_id: BackendId) -> Option<&BackendResource<<A as GpuAsset>::Loaded>> {
        self.loaded.get(&backend_id)
    }

    /// Loads the asset to the GPU, if it isn't already.
    pub fn load(
        &mut self,
        asset: &A,
        context: &LoadContext,
        cache: &mut BackendResourceCache,
    ) -> Result<(), Error> {
        match self.loaded.entry(context.backend.id()) {
            linear_map::Entry::Occupied(_occupied) => {}
            linear_map::Entry::Vacant(vacant) => {
                tracing::debug!(asset_type = type_name::<A>(), backend_id = ?context.backend.id(), "loading asset to gpu");

                let resource = if let Some(asset_id) = asset.maybe_asset_id() {
                    // BackendResource wraps the Arc, but we need to pass an Arc to the cache, so we
                    // take out the inner and later put it back in
                    let inner = cache
                        .cache
                        .get_or_try_insert((asset_id, context.backend.id()), || {
                            Ok::<_, Error>(BackendResource::new(asset.load(context)?).inner)
                        })?;
                    BackendResource { inner }
                }
                else {
                    BackendResource::new(asset.load(context)?)
                };

                vacant.insert(resource);
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct LoadContext<'a> {
    pub backend: &'a Backend,
    pub pipeline: &'a Pipeline,
}

pub struct BackendResourceId<R> {
    id: NonZeroUsize,
    _t: PhantomData<R>,
}

unsafe impl<R> Send for BackendResourceId<R> {}
unsafe impl<R> Sync for BackendResourceId<R> {}

impl<R> PartialEq for BackendResourceId<R> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<R> Eq for BackendResourceId<R> {}

impl<R> PartialOrd for BackendResourceId<R> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<R> Ord for BackendResourceId<R> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl<R> Hash for BackendResourceId<R> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// todo: clippy is going to complain that this is Copy, but has a 'non-trivial'
// Clone impl. what is it going to suggest?
impl<R> Clone for BackendResourceId<R> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _t: PhantomData,
        }
    }
}

impl<R> Copy for BackendResourceId<R> {}

impl<R> Debug for BackendResourceId<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackendResourceId")
            .field("id", &self.id)
            .field("R", &type_name::<R>())
            .finish()
    }
}

impl<R> BackendResourceId<R> {
    fn new() -> Self {
        static IDS: AtomicUsize = AtomicUsize::new(1);
        let id = IDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self {
            id: NonZeroUsize::new(id).unwrap(),
            _t: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct BackendResource<R> {
    inner: Arc<BackendResourceInner<R>>,
}

#[derive(Debug)]
struct BackendResourceInner<R> {
    id: BackendResourceId<R>,
    resource: ThreadLocalCell<R>,
}

impl<R> BackendResource<R> {
    pub fn new(resource: R) -> Self {
        Self {
            inner: Arc::new(BackendResourceInner {
                id: BackendResourceId::new(),
                resource: ThreadLocalCell::new(resource),
            }),
        }
    }

    pub fn id(&self) -> BackendResourceId<R> {
        self.inner.id
    }

    pub fn get(&self) -> &R {
        self.inner.resource.get()
    }
}

impl<R> Clone for BackendResource<R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<R> PartialEq for BackendResource<R> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.id == other.inner.id
    }
}

impl<R> Eq for BackendResource<R> {}

impl<R> PartialOrd for BackendResource<R> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<R> Ord for BackendResource<R> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.id.cmp(&other.inner.id)
    }
}

impl<R> Hash for BackendResource<R> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.id.hash(state);
    }
}

impl<R> From<R> for BackendResource<R> {
    fn from(resource: R) -> Self {
        Self::new(resource)
    }
}

#[derive(Debug, Default)]
pub struct BackendResourceCache {
    cache: AnyArcCache<(AssetId, BackendId)>,
}

/// Loads assets to GPU(s)
#[derive(Default)]
pub struct GpuLoadingSystem {
    command_buffer: hecs::CommandBuffer,
}

impl System for GpuLoadingSystem {
    type Error = Error;

    fn label(&self) -> &'static str {
        "gpu-loading"
    }

    fn poll_system(
        &mut self,
        _task_context: &mut std::task::Context<'_>,
        system_context: &mut SystemContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        /// Queries assets and loads them to a GPU
        fn load<A: GpuAsset + Send + Sync + 'static>(
            load_context: &LoadContext,
            world: &hecs::World,
            command_buffer: &mut hecs::CommandBuffer,
            cache: &mut BackendResourceCache,
        ) -> Result<(), Error> {
            let mut query = world.query::<(&A, Option<&mut OnGpu<A>>)>();
            for (entity, (asset, on_gpu)) in &mut query {
                let mut on_gpu_buf = None;

                let on_gpu = on_gpu.unwrap_or_else(|| {
                    on_gpu_buf = Some(OnGpu::default());
                    on_gpu_buf.as_mut().unwrap()
                });

                on_gpu.load(&asset, &load_context, cache)?;

                if let Some(on_gpu) = on_gpu_buf {
                    command_buffer.insert_one(entity, on_gpu);
                }
            }

            Ok(())
        }

        let cache = system_context
            .resources
            .get_mut_or_insert_default::<BackendResourceCache>();

        let mut render_targets = system_context.world.query::<&RenderTarget>();

        // for each RenderTarget, load assets to its backend
        //
        // this could be done more efficiently, especially if `RenderTarget`s share
        // backends (e.g. on WEBGPU)
        //
        // todo: don't just load everything onto every backend -.-
        for (_entity, render_target) in &mut render_targets {
            let render_target = render_target.inner.get();
            let load_context = LoadContext {
                backend: &render_target.backend,
                pipeline: &render_target.pipeline,
            };

            load::<Mesh>(
                &load_context,
                &system_context.world,
                &mut self.command_buffer,
                cache,
            )?;
            load::<Material>(
                &load_context,
                &system_context.world,
                &mut self.command_buffer,
                cache,
            )?;
        }

        drop(render_targets);

        self.command_buffer.run_on(&mut system_context.world);

        Poll::Ready(Ok(()))
    }
}

impl Debug for GpuLoadingSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuLoadingSystem").finish_non_exhaustive()
    }
}
