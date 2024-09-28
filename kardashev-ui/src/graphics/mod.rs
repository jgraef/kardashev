pub mod camera;
pub mod event_loop;
pub mod material;
pub mod mesh;
pub mod model;
pub mod renderer;
pub mod texture;
pub mod transform;
pub mod window;

use std::{
    collections::HashMap,
    sync::{
        atomic::{
            AtomicUsize,
            Ordering,
        },
        Arc,
    },
};

use image::RgbaImage;
use renderer::{
    CreateRendererContext,
    RenderContext,
    RenderPlugin,
    Renderer,
    ResizeContext,
};
use tokio::sync::oneshot;
use web_sys::HtmlCanvasElement;
use wgpu::{
    util::DeviceExt,
    TextureViewDescriptor,
};
use window::WindowHandler;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event_loop::{
        ActiveEventLoop,
        EventLoop,
        EventLoopProxy,
    },
    window::{
        Window,
        WindowAttributes,
        WindowId,
    },
};

use self::texture::Texture;
use crate::utils::spawn_local_and_handle_error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no backends")]
    NoBackends,

    #[error("no adapter")]
    NoAdapter,

    #[error("failed to request device")]
    RequestDevice(#[from] wgpu::RequestDeviceError),

    #[error("no such window: {0:?}")]
    NoSuchWindow(WindowId),
}

enum Command {
    CreateWindow {
        canvas: HtmlCanvasElement,
        instance: Arc<wgpu::Instance>,
        tx_response: oneshot::Sender<CreateWindowResponse>,
    },
    InitializeWindow {
        window: Arc<Window>,
        surface: wgpu::Surface<'static>,
        initial_size: PhysicalSize<u32>,
        render_context: Option<Backend>,
        window_handler: Box<dyn WindowHandler>,
        render_plugin: Box<dyn RenderPlugin>,
    },
    DestroyWindow {
        window_id: WindowId,
    },
    LoadTexture {
        window_id: WindowId,
        image: RgbaImage,
        label: Option<String>,
        tx_response: oneshot::Sender<Result<Texture, Error>>,
    },
}

struct CreateWindowResponse {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
}

#[derive(Clone, Debug, Default)]
pub struct RendererConfig {
    pub power_preference: wgpu::PowerPreference,
    pub backend_type: BackendType,
}

#[derive(Clone)]
pub struct Graphics {
    config: Arc<RendererConfig>,
    proxy: EventLoopProxy<Command>,
}

impl Graphics {
    pub fn new(config: RendererConfig) -> Self {
        let config = Arc::new(config);

        let event_loop = EventLoop::with_user_event()
            .build()
            .expect("failed to create event loop");
        let proxy = event_loop.create_proxy();

        let renderer = Self {
            config: config.clone(),
            proxy,
        };

        spawn_local_and_handle_error::<_, Error>({
            let renderer = renderer.clone();
            async move {
                #[allow(unused_mut, unused_variables)]
                let mut render_loop = MainLoop::new(renderer, config).await?;

                #[cfg(target_arch = "wasm32")]
                {
                    use winit::platform::web::EventLoopExtWebSys;
                    tracing::debug!("spawning window event loop");
                    event_loop.spawn_app(render_loop);
                }

                Ok(())
            }
        });

        renderer
    }

    fn send_command(&self, command: Command) {
        let _ = self.proxy.send_event(command);
    }

    pub async fn create_window(
        &self,
        canvas: HtmlCanvasElement,
        window_handler: Box<dyn WindowHandler>,
        render_plugin: Box<dyn RenderPlugin>,
    ) -> window::Window {
        let (tx_response, rx_response) = oneshot::channel();

        tracing::debug!("creating WebGL instance");
        let instance = Arc::new(wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        }));

        let initial_size = canvas_size(&canvas);

        self.send_command(Command::CreateWindow {
            canvas,
            instance: instance.clone(),
            tx_response,
        });
        let CreateWindowResponse { window, surface } =
            rx_response.await.expect("tx_response dropped");

        let render_context = match self.config.backend_type {
            BackendType::WebGpu => None,
            BackendType::WebGl => {
                Some(
                    Backend::new(instance, &self.config, Some(&surface))
                        .await
                        .expect("todo: handle error"),
                )
            }
        };

        self.send_command(Command::InitializeWindow {
            window: window.clone(),
            surface,
            initial_size,
            render_context,
            window_handler,
            render_plugin,
        });

        let window = window::Window::new(self.clone(), window);

        window
    }

    pub fn destroy_window(&self, window_id: WindowId) {
        self.send_command(Command::DestroyWindow { window_id });
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum BackendType {
    WebGpu,
    #[default]
    WebGl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BackendId(usize);

#[derive(Debug)]
pub(super) struct Backend {
    id: BackendId,
    instance: Arc<wgpu::Instance>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Backend {
    async fn webgpu_shared(config: &RendererConfig) -> Result<Self, Error> {
        tracing::debug!("creating WEBGPU instance");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });
        Self::new(Arc::new(instance), config, None).await
    }

    async fn new(
        instance: Arc<wgpu::Instance>,
        config: &RendererConfig,
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
            tracing::error!(%error, "uncaptured error");
        }));

        static IDS: AtomicUsize = AtomicUsize::new(1);
        let id = BackendId(IDS.fetch_add(1, Ordering::Relaxed));

        Ok(Self {
            id,
            instance,
            adapter,
            device,
            queue,
        })
    }
}

struct MainLoop {
    config: Arc<RendererConfig>,
    renderer: Graphics,
    shared_backend: Option<Arc<Backend>>,
    windows: HashMap<WindowId, WindowState>,
}

impl MainLoop {
    async fn new(renderer: Graphics, config: Arc<RendererConfig>) -> Result<Self, Error> {
        let shared_backend = match config.backend_type {
            BackendType::WebGpu => {
                // WebGPU (and really all backends except webgl can reuse the context)
                Some(Arc::new(Backend::webgpu_shared(&config).await?))
            }
            BackendType::WebGl => {
                // for WebGL we have to create a render context for each surface
                None
            }
        };

        Ok(Self {
            config,
            renderer,
            shared_backend,
            windows: Default::default(),
        })
    }

    fn create_window(
        &self,
        canvas: HtmlCanvasElement,
        instance: Arc<wgpu::Instance>,
        tx_response: oneshot::Sender<CreateWindowResponse>,
        active_event_loop: &ActiveEventLoop,
    ) {
        #[allow(unused_mut)]
        let mut window_attributes = WindowAttributes::default();
        #[allow(unused_variables)]
        let canvas = canvas;

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowAttributesExtWebSys;
            window_attributes = window_attributes.with_canvas(Some(canvas));
        }
        let window = active_event_loop
            .create_window(window_attributes)
            .expect("failed to create window");
        let window = Arc::new(window);

        let surface = instance
            .create_surface(window.clone())
            .expect("failed to create surface");

        let _ = tx_response.send(CreateWindowResponse { window, surface });
    }
}

impl ApplicationHandler<Command> for MainLoop {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        tracing::debug!(?window_id, ?event, "window event");

        if let Some(window) = self.windows.get_mut(&window_id) {
            match event {
                winit::event::WindowEvent::Resized(physical_size) => {
                    tracing::debug!(?physical_size, "window resize");
                    if physical_size.width > 0 && physical_size.height > 0 {
                        window.surface_config.width = physical_size.width;
                        window.surface_config.height = physical_size.height;
                        window
                            .surface
                            .configure(&window.backend.device, &window.surface_config);
                    }
                    window.renderer.resize(ResizeContext {
                        device: &window.backend.device,
                        new_size: physical_size,
                    });
                    window.window_handler.on_resize(physical_size);
                }
                winit::event::WindowEvent::RedrawRequested => {
                    let target_texture = window
                        .surface
                        .get_current_texture()
                        .expect("get_current_texture failed");

                    window.renderer.render(RenderContext {
                        target_texture,
                        window: &window.window,
                        device: &window.backend.device,
                        queue: &window.backend.queue,
                    });
                }
                _ => {}
            }
        }
    }

    fn user_event(&mut self, active_event_loop: &ActiveEventLoop, command: Command) {
        match command {
            Command::CreateWindow {
                canvas,
                instance,
                tx_response,
            } => self.create_window(canvas, instance, tx_response, active_event_loop),
            Command::InitializeWindow {
                window,
                surface,
                initial_size,
                render_context: backend,
                window_handler,
                render_plugin,
            } => {
                let backend = backend
                    .map(Arc::new)
                    .or_else(|| self.shared_backend.clone())
                    .expect("missing render context");

                self.windows.insert(
                    window.id(),
                    WindowState::new(
                        window,
                        surface,
                        initial_size,
                        backend,
                        window_handler,
                        render_plugin,
                    ),
                );
            }
            Command::DestroyWindow { window_id } => {
                if let Some(_window) = self.windows.remove(&window_id) {
                    // we think we can just drop everything
                }
            }
            Command::LoadTexture {
                window_id,
                image,
                label,
                tx_response,
            } => {
                let texture = self
                    .windows
                    .get(&window_id)
                    .map(|window| window.load_texture(image, label.as_deref()))
                    .ok_or_else(|| Error::NoSuchWindow(window_id));
                let _ = tx_response.send(texture);
            }
        }
    }
}

struct WindowState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    backend: Arc<Backend>,
    window_handler: Box<dyn WindowHandler>,
    renderer: Box<dyn Renderer>,
}

impl WindowState {
    fn new(
        window: Arc<Window>,
        surface: wgpu::Surface<'static>,
        initial_size: PhysicalSize<u32>,
        backend: Arc<Backend>,
        window_handler: Box<dyn WindowHandler>,
        render_plugin: Box<dyn RenderPlugin>,
    ) -> Self {
        // configure surface
        let surface_caps = surface.get_capabilities(&backend.adapter);

        // Shader code in this tutorial assumes an sRGB surface texture. Using a
        // different one will result in all the colors
        // coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .filter(|f| f.is_srgb())
            .next()
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: initial_size.width,
            height: initial_size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&backend.device, &surface_config);

        // create renderer
        let renderer = render_plugin.create_renderer(CreateRendererContext {
            surface_config: &surface_config,
            surface_size: initial_size,
            device: &backend.device,
        });

        WindowState {
            window,
            surface,
            surface_config,
            backend,
            window_handler,
            renderer,
        }
    }

    fn load_texture(&self, image: RgbaImage, label: Option<&str>) -> Texture {
        let image_size = image.dimensions();
        let texture_size = wgpu::Extent3d {
            width: image_size.0,
            height: image_size.1,
            depth_or_array_layers: 1,
        };

        let texture = self.backend.device.create_texture_with_data(
            &self.backend.queue,
            &wgpu::TextureDescriptor {
                label,
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::default(),
            image.as_raw(),
        );

        let view = texture.create_view(&TextureViewDescriptor {
            label,
            ..Default::default()
        });
        let sampler = self
            .backend
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                label,
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

        Texture {
            texture: Arc::new(texture),
            view: Arc::new(view),
            sampler: Arc::new(sampler),
        }
    }
}

fn canvas_size(canvas: &HtmlCanvasElement) -> PhysicalSize<u32> {
    let width = std::cmp::max(1, canvas.width());
    let height = std::cmp::max(1, canvas.height());
    PhysicalSize::new(width, height)
}
