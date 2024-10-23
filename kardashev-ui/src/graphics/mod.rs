pub mod backend;
pub mod blinn_phong;
pub mod camera;
pub mod draw_batch;
pub mod hdr;
pub mod light;
pub mod material;
pub mod mesh;
pub mod model;
pub mod pbr;
pub mod render_3d;
pub mod render_frame;
pub mod texture;
pub mod transform;
pub mod utils;

use std::{
    fmt::Debug,
    num::NonZeroU32,
    sync::{
        atomic::{
            AtomicU32,
            Ordering,
        },
        Arc,
    },
};

use serde::{
    Deserialize,
    Serialize,
};
use tokio::sync::{
    mpsc,
    oneshot,
    watch,
};
use web_sys::HtmlCanvasElement;

use crate::{
    assets::system::AssetTypeRegistry,
    ecs::plugin::{
        Plugin,
        RegisterPluginContext,
    },
    graphics::{
        backend::{
            Backend,
            BackendType,
        },
        blinn_phong::BlinnPhongMaterial,
        material::Material,
        mesh::Mesh,
        pbr::PbrMaterial,
        render_frame::rendering_system,
        texture::Texture,
        transform::local_to_global_transform_system,
        utils::GpuResourceCache,
    },
    utils::{
        futures::spawn_local_and_handle_error,
        thread_local_cell::ThreadLocalError,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no backends")]
    NoBackends,

    #[error("no adapter")]
    NoAdapter,

    #[error("failed to request device")]
    RequestDevice(#[source] ThreadLocalError<wgpu::RequestDeviceError>),

    #[error("failed to create surface")]
    CreateSurface(#[source] ThreadLocalError<wgpu::CreateSurfaceError>),
}

impl From<wgpu::RequestDeviceError> for Error {
    fn from(error: wgpu::RequestDeviceError) -> Self {
        Self::RequestDevice(ThreadLocalError::new(error))
    }
}

impl From<wgpu::CreateSurfaceError> for Error {
    fn from(error: wgpu::CreateSurfaceError) -> Self {
        Self::CreateSurface(ThreadLocalError::new(error))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub backend_type: SelectBackendType,
    pub power_preference: wgpu::PowerPreference,
    pub memory_hints: MemoryHints,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SelectBackendType {
    #[default]
    AutoDetect,
    Select(BackendType),
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum MemoryHints {
    #[default]
    Performance,
    MemoryUsage,
}

impl MemoryHints {
    pub fn as_wgpu(&self) -> wgpu::MemoryHints {
        match self {
            Self::Performance => wgpu::MemoryHints::Performance,
            Self::MemoryUsage => wgpu::MemoryHints::MemoryUsage,
        }
    }
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
            reactor.run().await;
            Ok::<(), Error>(())
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

        Ok(Surface {
            backend,
            surface: Arc::new(surface),
            surface_configuration,
        })
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

                match Backend::new(Arc::new(instance), &config, None, wgpu::Limits::default()).await
                {
                    Ok(shared_backend) => (BackendType::WebGpu, Some(shared_backend)),
                    Err(error) => {
                        tracing::info!(
                            ?error,
                            "failed to initialize WEBGPU backend, falling back to WebGL"
                        );
                        (BackendType::WebGl, None)
                    }
                }
            }
            SelectBackendType::Select(backend_type) => {
                tracing::debug!(?backend_type, "initializing shared backend");
                let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                    backends: backend_type.as_wgpu(),
                    ..Default::default()
                });
                let shared_backend =
                    Backend::new(Arc::new(instance), &config, None, wgpu::Limits::default())
                        .await?;
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

    async fn run(mut self) {
        while let Some(command) = self.rx_command.recv().await {
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
        }
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

            let surface = instance.create_surface(window_handle)?;

            let backend = Backend::new(
                instance,
                &self.config,
                Some(&surface),
                wgpu::Limits::downlevel_webgl2_defaults(),
            )
            .await?;

            (surface, backend)
        };

        let surface_capabilities = surface.get_capabilities(&backend.adapter);

        tracing::debug!(
            "supported surface formats: {:#?}",
            surface_capabilities.formats
        );

        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        tracing::debug!("selected surface format: {surface_format:?}");

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    pub fn from_texture(texture: &wgpu::Texture) -> Self {
        Self {
            width: texture.width(),
            height: texture.height(),
        }
    }
}

#[derive(Debug)]
pub struct Surface {
    backend: Backend,
    surface: Arc<wgpu::Surface<'static>>,
    surface_configuration: wgpu::SurfaceConfiguration,
}

impl Surface {
    pub fn size(&self) -> SurfaceSize {
        SurfaceSize::from_surface_configuration(&self.surface_configuration)
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.surface_configuration.format
    }

    pub fn resize(&mut self, size: SurfaceSize) {
        self.surface_configuration.width = size.width;
        self.surface_configuration.height = size.height;
        self.surface
            .configure(&self.backend.device, &self.surface_configuration);
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
                .register::<Material<BlinnPhongMaterial>>()
                .register::<Material<PbrMaterial>>();
        }
        else {
            tracing::warn!("resource AssetTypeRegistry is missing. can't register asset types for rendering system");
        }

        context.resources.insert(GpuResourceCache::default());
        context
            .schedule
            .add_system(local_to_global_transform_system);
        context.schedule.add_system(rendering_system);
    }
}
