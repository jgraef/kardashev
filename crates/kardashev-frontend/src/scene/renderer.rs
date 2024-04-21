use std::{
    collections::HashMap,
    sync::{
        atomic::{
            AtomicUsize,
            Ordering,
        },
        Arc,
        Mutex,
    },
};

use leptos::{
    html::Canvas,
    HtmlElement,
};
use wgpu::{
    Adapter,
    Device,
    DeviceDescriptor,
    Instance,
    Limits,
    PowerPreference,
    Queue,
    RequestAdapterOptions,
    RequestAdapterOptionsBase,
    RequestDeviceError,
    Surface,
    SurfaceConfiguration,
    TextureUsages,
};
use winit::{
    dpi::PhysicalSize,
    event::Event,
    event_loop::{
        EventLoopBuilder,
        EventLoopProxy,
        EventLoopWindowTarget,
    },
    window::{
        Window,
        WindowBuilder,
        WindowId,
    },
};

use crate::utils::spawn_local_and_handle_error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no adapter")]
    NoAdapter,

    #[error("failed to request device")]
    RequestDevice(#[from] RequestDeviceError),
}

enum Command {
    CreateWindow {
        canvas: HtmlElement<Canvas>,
        event_handler: WindowEventHandler,
    },
    DestroyWindow {
        window_id: WindowId,
    },
}

#[derive(Clone)]
pub struct SceneRenderer {
    proxy: EventLoopProxy<Command>,
}

impl SceneRenderer {
    pub fn new() -> Self {
        let event_loop = EventLoopBuilder::with_user_event()
            .build()
            .expect("failed to create event loop");
        let proxy = event_loop.create_proxy();

        let scene_renderer = Self { proxy };

        spawn_local_and_handle_error::<_, Error>({
            let scene_renderer = scene_renderer.clone();
            async move {
                let mut state = RenderLoop::new(scene_renderer).await?;

                #[cfg(target_arch = "wasm32")]
                {
                    use winit::platform::web::EventLoopExtWebSys;
                    tracing::debug!("spawning window event loop");
                    window_loop.spawn(move |event, target| {
                        state.handle_event(event, target);
                    });
                }

                Ok(())
            }
        });

        scene_renderer
    }

    fn send_command(&self, command: Command) {
        let _ = self.proxy.send_event(command);
    }

    pub fn create_window(
        &self,
        canvas: HtmlElement<Canvas>,
        event_handler: impl FnMut(WindowEvent) + 'static,
    ) {
        self.send_command(Command::CreateWindow {
            canvas,
            event_handler: Box::new(event_handler),
        });
    }
}

struct RenderLoop {
    scene_renderer: SceneRenderer,
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    windows: HashMap<WindowId, WindowState>,
}

impl RenderLoop {
    async fn new(scene_renderer: SceneRenderer) -> Result<Self, Error> {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: Default::default(),
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .ok_or_else(|| Error::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Default::default(),
                    required_limits: Limits::downlevel_webgl2_defaults(),
                },
                None,
            )
            .await?;

        Ok(Self {
            scene_renderer,
            instance,
            adapter,
            device,
            queue,
            windows: Default::default(),
        })
    }

    fn handle_event(&mut self, event: Event<Command>, target: EventLoopWindowTarget<Command>) {
        match event {
            Event::WindowEvent { window_id, event } => {
                if let Some(window_state) = self.windows.get_mut(&window_id) {
                    match event {
                        winit::event::WindowEvent::Resized(physical_size) => {
                            tracing::debug!(?physical_size, "window resize");
                        }
                        winit::event::WindowEvent::RedrawRequested => todo!(),
                        _ => {}
                    }
                    //(window_state.event_handler)(&self.handle, event);
                }
            }
            Event::UserEvent(command) => {
                match command {
                    Command::CreateWindow {
                        canvas,
                        mut event_handler,
                    } => {
                        let size = PhysicalSize::new(canvas.width(), canvas.height());

                        let mut window_builder = WindowBuilder::new().with_inner_size(size);

                        #[cfg(target_arch = "wasm32")]
                        {
                            use winit::platform::web::WindowBuilderExtWebSys;
                            window_builder = window_builder.with_canvas(Some(canvas));
                        }

                        let window = window_builder
                            .build(&target)
                            .expect("failed to build window");
                        let window = Arc::new(window);
                        let window_id = window.id();

                        let surface = self
                            .instance
                            .create_surface(window.clone())
                            .expect("failed to create surface");

                        let surface_caps = surface.get_capabilities(&self.adapter);
                        // Shader code in this tutorial assumes an sRGB surface texture. Using a
                        // different one will result in all the colors
                        // coming out darker. If you want to support non
                        // sRGB surfaces, you'll need to account for that when drawing to the frame.
                        let surface_format = surface_caps
                            .formats
                            .iter()
                            .copied()
                            .filter(|f| f.is_srgb())
                            .next()
                            .unwrap_or(surface_caps.formats[0]);
                        let surface_config = SurfaceConfiguration {
                            usage: TextureUsages::RENDER_ATTACHMENT,
                            format: surface_format,
                            width: size.width,
                            height: size.height,
                            present_mode: surface_caps.present_modes[0],
                            alpha_mode: surface_caps.alpha_modes[0],
                            view_formats: vec![],
                            desired_maximum_frame_latency: 2,
                        };
                        surface.configure(&self.device, &surface_config);

                        let handle = WindowHandle::new(self.scene_renderer.clone(), window_id);

                        event_handler(WindowEvent::Created { handle });

                        self.windows.insert(
                            window_id,
                            WindowState {
                                window,
                                surface,
                                event_handler,
                            },
                        );
                    }
                    Command::DestroyWindow { window_id } => {
                        tracing::debug!(?window_id, "todo: destroy window")
                    }
                }
            }
            _ => {}
        }
    }
}

pub enum WindowEvent {
    Created { handle: WindowHandle },
}

type WindowEventHandler = Box<dyn FnMut(WindowEvent)>;

pub struct WindowHandle {
    scene_renderer: SceneRenderer,
    window_id: WindowId,
    reference_count: Arc<AtomicUsize>,
}

impl WindowHandle {
    fn new(scene_renderer: SceneRenderer, window_id: WindowId) -> Self {
        Self {
            scene_renderer,
            window_id,
            reference_count: Arc::new(AtomicUsize::new(1)),
        }
    }
}

impl Clone for WindowHandle {
    fn clone(&self) -> Self {
        self.reference_count.fetch_add(1, Ordering::Relaxed);
        Self {
            scene_renderer: self.scene_renderer.clone(),
            window_id: self.window_id,
            reference_count: self.reference_count.clone(),
        }
    }
}

impl Drop for WindowHandle {
    fn drop(&mut self) {
        if self.reference_count.fetch_sub(1, Ordering::Relaxed) <= 1 {
            self.scene_renderer.send_command(Command::DestroyWindow {
                window_id: self.window_id,
            });
        }
    }
}

struct WindowState {
    window: Arc<Window>,
    surface: Surface<'static>,
    event_handler: WindowEventHandler,
}
