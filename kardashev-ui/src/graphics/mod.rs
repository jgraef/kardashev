pub mod camera;
pub mod draw_batch;
pub mod loading;
pub mod material;
pub mod mesh;
pub mod model;
pub mod pipeline;
pub mod rendering_system;
pub mod texture;
pub mod transform;
pub mod util;

use std::{
    fmt::Debug,
    hash::Hash,
    num::{
        NonZeroU32,
        NonZeroUsize,
    },
    sync::{
        atomic::{
            AtomicU32,
            AtomicUsize,
            Ordering,
        },
        Arc,
    },
};

use loading::GpuLoadingSystem;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::sync::{
    mpsc,
    oneshot,
    watch,
};
use transform::LocalToGlobalTransformSystem;
use web_sys::HtmlCanvasElement;

use crate::{
    assets::system::AssetTypeRegistry,
    graphics::{
        loading::BackendResourceCache,
        material::Material,
        mesh::Mesh,
        rendering_system::RenderingSystem,
        texture::Texture,
    },
    utils::futures::spawn_local_and_handle_error,
    world::{
        Plugin,
        RegisterPluginContext,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no backends")]
    NoBackends,

    #[error("no adapter")]
    NoAdapter,

    #[error("failed to request device")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub power_preference: wgpu::PowerPreference,
    pub backend_type: SelectBackendType,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SelectBackendType {
    #[default]
    AutoDetect,
    Select(BackendType),
}

#[derive(Clone, Debug)]
pub struct Graphics {
    tx_command: mpsc::Sender<Command>,
}

impl Graphics {
    pub fn new(config: Config) -> Self {
        tracing::debug!(?config, "initializing graphics");

        let (tx_command, rx_command) = mpsc::channel(16);

        spawn_local_and_handle_error(async move {
            let reactor = Reactor::new(config, rx_command).await?;
            reactor.run().await
        });

        Self { tx_command }
    }

    async fn send_command(&self, command: Command) {
        self.tx_command
            .send(command)
            .await
            .expect("graphics reactor died");
    }

    pub async fn create_surface(
        &self,
        window_handle: WindowHandle,
        surface_size: SurfaceSize,
    ) -> Result<Surface, Error> {
        let (tx_result, rx_result) = oneshot::channel();

        self.send_command(Command::CreateSurface {
            window_handle,
            surface_size,
            tx_result,
        })
        .await;

        let CreateSurfaceResponse {
            backend,
            surface,
            surface_configuration,
        } = rx_result.await.unwrap()?;

        let (tx_resize, _) = watch::channel(surface_size);
        let (tx_visible, _) = watch::channel(true);

        Ok(Surface {
            backend,
            surface: Arc::new(surface),
            surface_configuration,
            tx_resize,
            tx_visible,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackendType {
    WebGpu,
    #[default]
    WebGl,
}

impl BackendType {
    fn uses_shared_backend(&self) -> bool {
        matches!(self, Self::WebGpu)
    }

    fn as_wgpu(&self) -> wgpu::Backends {
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
    id: BackendId,
    instance: Arc<wgpu::Instance>,
    adapter: Arc<wgpu::Adapter>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl Backend {
    async fn new(
        instance: Arc<wgpu::Instance>,
        config: &Config,
        compatible_surface: Option<&wgpu::Surface<'static>>,
    ) -> Result<Self, Error> {
        tracing::debug!("creating render adapter");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: config.power_preference,
                compatible_surface: compatible_surface,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| Error::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: Default::default(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    memory_hints: wgpu::MemoryHints::Performance,
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

    pub fn id(&self) -> BackendId {
        self.id
    }
}

#[derive(Debug)]
struct Reactor {
    config: Config,
    backend_type: BackendType,
    shared_backend: Option<Backend>,
    rx_command: mpsc::Receiver<Command>,
}

impl Reactor {
    async fn new(config: Config, rx_command: mpsc::Receiver<Command>) -> Result<Self, Error> {
        let (backend_type, shared_backend) = match config.backend_type {
            SelectBackendType::AutoDetect => {
                tracing::debug!("trying WEBGPU");
                let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::BROWSER_WEBGPU,
                    ..Default::default()
                });

                if let Ok(shared_backend) = Backend::new(Arc::new(instance), &config, None).await {
                    (BackendType::WebGpu, Some(shared_backend))
                }
                else {
                    tracing::info!("failed to initialize WEBGPU backend, falling back to WebGL");
                    (BackendType::WebGl, None)
                }
            }
            SelectBackendType::Select(backend_type) => {
                tracing::debug!(?backend_type, "initializing shared backend");
                let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                    backends: backend_type.as_wgpu(),
                    ..Default::default()
                });
                let shared_backend = Backend::new(Arc::new(instance), &config, None).await?;
                (backend_type, Some(shared_backend))
            }
        };

        Ok(Self {
            config,
            backend_type,
            shared_backend,
            rx_command,
        })
    }

    async fn run(mut self) -> Result<(), Error> {
        while let Some(command) = self.rx_command.recv().await {
            self.handle_command(command).await?;
        }

        Ok(())
    }

    async fn handle_command(&mut self, command: Command) -> Result<(), Error> {
        match command {
            Command::CreateSurface {
                window_handle,
                surface_size,
                tx_result,
            } => {
                let result = self.create_surface(window_handle, surface_size).await;
                let _ = tx_result.send(result);
            }
        }

        Ok(())
    }

    async fn create_surface(
        &self,
        window_handle: WindowHandle,
        surface_size: SurfaceSize,
    ) -> Result<CreateSurfaceResponse, Error> {
        tracing::info!(?window_handle, ?surface_size, "creating surface");

        let (surface, backend) = if self.backend_type.uses_shared_backend() {
            let backend = self
                .shared_backend
                .as_ref()
                .expect("expected a shared backend for WebGPU backend");
            let surface = backend
                .instance
                .create_surface(window_handle)
                .expect("failed to create surface");
            (surface, backend.clone())
        }
        else {
            tracing::debug!("creating WebGL instance");
            let instance = Arc::new(wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: self.backend_type.as_wgpu(),
                ..Default::default()
            }));

            let surface = instance
                .create_surface(window_handle)
                .expect("failed to create surface");

            let backend = Backend::new(instance, &self.config, Some(&surface))
                .await
                .expect("todo: handle error");

            (surface, backend)
        };

        let surface_capabilities = surface.get_capabilities(&backend.adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: surface_size.width,
            height: surface_size.height,
            present_mode: surface_capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&backend.device, &surface_configuration);

        Ok(CreateSurfaceResponse {
            backend,
            surface,
            surface_configuration,
        })
    }
}

#[derive(Debug)]
enum Command {
    CreateSurface {
        window_handle: WindowHandle,
        surface_size: SurfaceSize,
        tx_result: oneshot::Sender<Result<CreateSurfaceResponse, Error>>,
    },
}

#[derive(Debug)]
struct CreateSurfaceResponse {
    backend: Backend,
    surface: wgpu::Surface<'static>,
    surface_configuration: wgpu::SurfaceConfiguration,
}

#[derive(Clone, Copy, Debug)]
pub struct WindowHandle {
    id: NonZeroU32,
}

impl WindowHandle {
    pub fn new() -> Self {
        static IDS: AtomicU32 = AtomicU32::new(1);
        Self {
            id: NonZeroU32::new(IDS.fetch_add(1, Ordering::Relaxed)).unwrap(),
        }
    }

    pub fn id(&self) -> NonZeroU32 {
        self.id
    }
}

impl raw_window_handle::HasWindowHandle for WindowHandle {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'static>, raw_window_handle::HandleError> {
        let raw = raw_window_handle::RawWindowHandle::Web(raw_window_handle::WebWindowHandle::new(
            self.id.into(),
        ));
        let window_handle = unsafe { raw_window_handle::WindowHandle::borrow_raw(raw) };
        Ok(window_handle)
    }
}

impl raw_window_handle::HasDisplayHandle for WindowHandle {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'static>, raw_window_handle::HandleError> {
        Ok(raw_window_handle::DisplayHandle::web())
    }
}

impl leptos::IntoAttribute for WindowHandle {
    fn into_attribute(self) -> leptos::Attribute {
        leptos::Attribute::String(self.id.to_string().into())
    }

    fn into_attribute_boxed(self: Box<Self>) -> leptos::Attribute {
        self.into_attribute()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SurfaceSize {
    pub width: u32,
    pub height: u32,
}

impl SurfaceSize {
    pub fn from_html_canvas(canvas: &HtmlCanvasElement) -> Self {
        Self {
            width: canvas.width().max(1),
            height: canvas.height().max(1),
        }
    }

    pub fn from_surface_configuration(surface_configuration: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            width: surface_configuration.width,
            height: surface_configuration.height,
        }
    }
}

#[derive(Debug)]
pub struct Surface {
    backend: Backend,
    surface: Arc<wgpu::Surface<'static>>,
    surface_configuration: wgpu::SurfaceConfiguration,
    tx_resize: watch::Sender<SurfaceSize>,
    tx_visible: watch::Sender<bool>,
}

impl Surface {
    pub fn resize(&mut self, surface_size: SurfaceSize) {
        self.surface_configuration.width = surface_size.width;
        self.surface_configuration.height = surface_size.height;
        tracing::info!(backend = ?self.backend.id, ?surface_size, "reconfiguring surface");
        let _ = self.tx_resize.send(surface_size);
        self.surface
            .configure(&self.backend.device, &self.surface_configuration);
    }

    pub fn set_visible(&mut self, visible: bool) {
        let _ = self.tx_visible.send(visible);
    }

    pub fn size_listener(&self) -> SurfaceSizeListener {
        SurfaceSizeListener {
            rx_resize: self.tx_resize.subscribe(),
        }
    }

    pub fn visibility_listener(&self) -> SurfaceVisibilityListener {
        SurfaceVisibilityListener {
            rx_visible: self.tx_visible.subscribe(),
        }
    }

    pub fn size(&self) -> SurfaceSize {
        self.tx_resize.borrow().clone()
    }
}

#[derive(Clone, Debug)]
pub struct SurfaceSizeListener {
    rx_resize: watch::Receiver<SurfaceSize>,
}

impl SurfaceSizeListener {
    pub fn get(&self) -> SurfaceSize {
        self.rx_resize.borrow().clone()
    }

    pub fn poll(&mut self) -> Option<SurfaceSize> {
        self.rx_resize
            .has_changed()
            .unwrap_or_default()
            .then(|| self.rx_resize.borrow_and_update().clone())
    }

    pub async fn wait(&mut self) -> Option<SurfaceSize> {
        Some(self.rx_resize.wait_for(|_| true).await.ok()?.clone())
    }
}

#[derive(Clone, Debug)]
pub struct SurfaceVisibilityListener {
    rx_visible: watch::Receiver<bool>,
}

impl SurfaceVisibilityListener {
    pub fn is_visible(&self) -> bool {
        self.rx_visible.borrow().clone()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn register(self, context: RegisterPluginContext) {
        if let Some(asset_type_registry) = context.resources.get_mut::<AssetTypeRegistry>() {
            asset_type_registry
                .register::<Texture>()
                .register::<Mesh>()
                .register::<Material>();
        }
        else {
            tracing::warn!("resource AssetTypeRegistry is missing. can't register asset types for rendering system");
        }

        context.resources.insert(BackendResourceCache::default());

        context
            .scheduler
            .add_update_system(LocalToGlobalTransformSystem);
        context
            .scheduler
            .add_render_system(GpuLoadingSystem::default());
        context.scheduler.add_render_system(RenderingSystem);
    }
}
