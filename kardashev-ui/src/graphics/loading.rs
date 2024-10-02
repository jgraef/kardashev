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
};

use linear_map::LinearMap;

use super::{
    rendering_system::LoadContext,
    BackendId,
    Error,
};
use crate::utils::thread_local_cell::ThreadLocalCell;

/// Trait defining how an [`Asset`] can be loaded to the GPU.
///
/// The associated constant [`Self::Loaded`] should contain the GPU handles.
/// This will be put into a map with on entry per backend (GPU) and then used by
/// the rendering pipeline.
pub trait GpuAsset {
    type Loaded;

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
    /// Returns the [`BackendResourceHandle`] for the backend specified by
    /// `context`.
    ///
    /// This will try to load the asset to the GPU, if it isn't already.
    pub fn get(
        &mut self,
        asset: &A,
        context: &LoadContext,
    ) -> Result<&BackendResource<<A as GpuAsset>::Loaded>, Error> {
        match self.loaded.entry(context.backend.id()) {
            linear_map::Entry::Occupied(occupied) => Ok(occupied.into_mut()),
            linear_map::Entry::Vacant(vacant) => {
                let resource = asset.load(context)?;
                Ok(vacant.insert(BackendResource::new(resource)))
            }
        }
    }
}

pub struct BackendResourceId<R> {
    id: NonZeroUsize,
    _t: PhantomData<R>,
}

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

/// Container that holds the backend resource's ID and wraps the resource in an
/// `Arc<ThreadLocalCell<_>>`, such that it is `Send + Sync` and can be cloned.
#[derive(Debug)]
pub struct BackendResource<R> {
    id: BackendResourceId<R>,
    resource: Arc<ThreadLocalCell<R>>,
}

impl<R> BackendResource<R> {
    pub fn new(resource: R) -> Self {
        Self {
            id: BackendResourceId::new(),
            resource: Arc::new(ThreadLocalCell::new(resource)),
        }
    }
}

impl<R> PartialEq for BackendResource<R> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
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
        self.id.cmp(&other.id)
    }
}

impl<R> Hash for BackendResource<R> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<R> Clone for BackendResource<R> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            resource: self.resource.clone(),
        }
    }
}

impl<R> BackendResource<R> {
    pub fn id(&self) -> BackendResourceId<R> {
        self.id
    }

    pub fn get(&self) -> &R {
        self.resource.get()
    }
}

impl<R> From<R> for BackendResource<R> {
    fn from(resource: R) -> Self {
        Self::new(resource)
    }
}
