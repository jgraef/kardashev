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

use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    graphics::{
        Config,
        Error,
    },
    utils::small_linear_map::{
        self,
        SmallLinearMap,
    },
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

#[derive(Clone, Debug)]
pub struct PerBackend<T> {
    map: SmallLinearMap<2, BackendId, T>,
}

impl<T> Default for PerBackend<T> {
    fn default() -> Self {
        Self {
            map: SmallLinearMap::new(),
        }
    }
}

impl<T> PerBackend<T> {
    pub fn get_or_try_insert<F, E>(&mut self, backend_id: BackendId, insert: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::error::Error + 'static,
    {
        match self.map.entry(backend_id) {
            small_linear_map::Entry::Occupied(occupied) => Ok(occupied.into_mut()),
            small_linear_map::Entry::Vacant(vacant) => {
                let value = insert()?;
                Ok(vacant.insert(value))
            }
        }
    }
}
