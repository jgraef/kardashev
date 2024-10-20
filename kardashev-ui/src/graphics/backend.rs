use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{
            AtomicUsize,
            Ordering,
        },
        Arc,
    },
};

use linear_map::LinearMap;
use serde::{
    Deserialize,
    Serialize,
};

use crate::graphics::{
    Config,
    Error,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackendType {
    WebGpu,
    #[default]
    WebGl,
}

impl BackendType {
    pub fn uses_shared_backend(&self) -> bool {
        matches!(self, Self::WebGpu)
    }

    pub fn as_wgpu(&self) -> wgpu::Backends {
        match self {
            BackendType::WebGpu => wgpu::Backends::BROWSER_WEBGPU,
            BackendType::WebGl => wgpu::Backends::GL,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BackendId(NonZeroUsize);

#[derive(Clone, Debug)]
pub struct Backend {
    pub id: BackendId,
    pub instance: Arc<wgpu::Instance>,
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

impl Backend {
    pub(super) async fn new(
        instance: Arc<wgpu::Instance>,
        config: &Config,
        compatible_surface: Option<&wgpu::Surface<'static>>,
        required_limits: wgpu::Limits,
    ) -> Result<Self, Error> {
        tracing::debug!("creating render adapter");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: config.power_preference,
                compatible_surface,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| Error::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: Default::default(),
                    required_limits,
                    memory_hints: config.memory_hints.as_wgpu(),
                },
                None,
            )
            .await?;

        device.on_uncaptured_error(Box::new(|error| {
            tracing::error!(%error, "uncaptured wgpu error");
            panic!("uncaptured wgpu error: {error}");
        }));

        tracing::debug!("device features: {:#?}", device.features());

        static IDS: AtomicUsize = AtomicUsize::new(1);
        let id = BackendId(NonZeroUsize::new(IDS.fetch_add(1, Ordering::Relaxed)).unwrap());

        Ok(Self {
            id,
            instance,
            adapter: Arc::new(adapter),
            device: Arc::new(device),
            queue: Arc::new(queue),
        })
    }
}

/// Struct holding the handles to on-GPU resources.
///
/// This is usually used as an component (e.g. as a `OnGpu<Mesh>`).
#[derive(Clone, Debug)]
pub struct PerBackend<T> {
    // todo: use SmallVec
    map: LinearMap<BackendId, T>,
}

impl<T> Default for PerBackend<T> {
    fn default() -> Self {
        Self {
            map: LinearMap::new(),
        }
    }
}

impl<T> PerBackend<T> {
    /// Returns the [`BackendResourceHandle`] for the specified `backend_id`.
    ///
    /// Loads the asset to the GPU, if it isn't already.
    pub fn get_or_try_insert<F, E>(&mut self, backend_id: BackendId, insert: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::error::Error + 'static,
    {
        let resource = match self.map.entry(backend_id) {
            linear_map::Entry::Occupied(occupied) => occupied.into_mut(),
            linear_map::Entry::Vacant(vacant) => {
                tracing::debug!(?backend_id, "loading asset to gpu");
                let resource = insert()?;
                vacant.insert(resource)
            }
        };

        Ok(resource)
    }
}
