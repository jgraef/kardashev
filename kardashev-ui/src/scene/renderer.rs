use std::{
    collections::HashMap,
    sync::Arc,
};

use bytemuck::{
    Pod,
    Zeroable,
};
use hecs::Entity;
use image::RgbaImage;
use palette::Srgba;
use tokio::sync::{
    mpsc,
    oneshot,
};
use web_sys::HtmlCanvasElement;
use wgpu::{
    util::DeviceExt,
    Instance,
    Surface,
};
use winit::{
    dpi::PhysicalSize,
    event::Event,
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

use super::{
    camera::Camera,
    mesh::Mesh,
    transform::Transform,
    window,
    Scene,
};
use crate::utils::spawn_local_and_handle_error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no backends")]
    NoBackends,

    #[error("no adapter")]
    NoAdapter,

    #[error("failed to request device")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
}

enum Command {
    CreateWindow {
        canvas: HtmlCanvasElement,
        instance: Arc<Instance>,
        tx_response: oneshot::Sender<CreateWindowResponse>,
    },
    InitializeWindow {
        window: Arc<Window>,
        surface: Surface<'static>,
        initial_size: PhysicalSize<u32>,
        scene_view: Option<SceneView>,
        tx_events: mpsc::Sender<window::Event>,
        render_context: Option<RenderContext>,
    },
    DestroyWindow {
        window_id: WindowId,
    },
    SetSceneView {
        window_id: WindowId,
        scene_view: Option<SceneView>,
    },
    CreateTexture {
        image: RgbaImage,
    },
}

struct CreateWindowResponse {
    window: Arc<Window>,
    surface: Surface<'static>,
}

#[derive(Clone, Debug, Default)]
pub struct SceneRendererConfig {
    pub power_preference: wgpu::PowerPreference,
    pub backend_type: BackendType,
}

#[derive(Clone)]
pub struct SceneRenderer {
    config: Arc<SceneRendererConfig>,
    proxy: EventLoopProxy<Command>,
}

impl SceneRenderer {
    pub fn new(config: SceneRendererConfig) -> Self {
        let config = Arc::new(config);

        let event_loop = EventLoop::with_user_event()
            .build()
            .expect("failed to create event loop");
        let proxy = event_loop.create_proxy();

        let scene_renderer = Self {
            config: config.clone(),
            proxy,
        };

        spawn_local_and_handle_error::<_, Error>({
            let scene_renderer = scene_renderer.clone();
            async move {
                #[allow(unused_mut, unused_variables)]
                let mut state = RenderLoop::new(scene_renderer, config).await?;

                #[cfg(target_arch = "wasm32")]
                {
                    use winit::platform::web::EventLoopExtWebSys;
                    tracing::debug!("spawning window event loop");
                    event_loop.spawn(move |event, target| {
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

    pub async fn create_window(
        &self,
        canvas: HtmlCanvasElement,
        scene_view: Option<SceneView>,
    ) -> (window::Window, window::Events) {
        let (tx_response, rx_response) = oneshot::channel();
        let (tx_events, rx_events) = mpsc::channel(32);

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
            rx_response.await.expect("tx_window dropped");

        let render_context = match self.config.backend_type {
            BackendType::WebGpu => None,
            BackendType::WebGl => {
                Some(
                    RenderContext::new(instance, &self.config, Some(&surface))
                        .await
                        .expect("todo: handle error"),
                )
            }
        };

        self.send_command(Command::InitializeWindow {
            window: window.clone(),
            surface,
            initial_size,
            scene_view,
            tx_events,
            render_context,
        });

        let window = window::Window::new(self.clone(), window);
        let events = window::Events::new(rx_events);

        (window, events)
    }

    pub(super) fn destroy_window(&self, window_id: WindowId) {
        self.send_command(Command::DestroyWindow { window_id });
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum BackendType {
    WebGpu,
    #[default]
    WebGl,
}

#[derive(Debug)]
pub(super) struct RenderContext {
    instance: Arc<wgpu::Instance>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl RenderContext {
    async fn webgpu_shared(config: &SceneRendererConfig) -> Result<Self, Error> {
        tracing::debug!("creating WEBGPU instance");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });
        Self::new(Arc::new(instance), config, None).await
    }

    async fn new(
        instance: Arc<Instance>,
        config: &SceneRendererConfig,
        compatible_surface: Option<&Surface<'static>>,
    ) -> Result<Self, Error> {
        tracing::debug!("creating render adapter");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: config.power_preference,
                // FIXME
                // we need to set a surface here (a MUST for WebGL), but to create the surface, we
                // need the window first. we can split window creation into several
                // steps:
                // 1. create the window and nothing else (via command to the event loop)
                // 2. create render context (instance, adapter, etc.), if using WebGL
                // 3. create surface (i.e. connect wgpu to the window)
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

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }
}

struct RenderLoop {
    config: Arc<SceneRendererConfig>,
    scene_renderer: SceneRenderer,
    shared_context: Option<Arc<RenderContext>>,
    windows: HashMap<WindowId, WindowState>,
}

impl RenderLoop {
    async fn new(
        scene_renderer: SceneRenderer,
        config: Arc<SceneRendererConfig>,
    ) -> Result<Self, Error> {
        let shared_context = match config.backend_type {
            BackendType::WebGpu => {
                // WebGPU (and really all backends except webgl can reuse the context)
                Some(Arc::new(RenderContext::webgpu_shared(&config).await?))
            }
            BackendType::WebGl => {
                // for WebGL we have to create a render context for each surface
                None
            }
        };

        Ok(Self {
            config,
            scene_renderer,
            shared_context,
            windows: Default::default(),
        })
    }

    fn handle_event(&mut self, event: Event<Command>, active_event_loop: &ActiveEventLoop) {
        match event {
            Event::WindowEvent { window_id, event } => {
                tracing::debug!(?window_id, ?event, "window event");

                if let Some(window) = self.windows.get_mut(&window_id) {
                    match event {
                        winit::event::WindowEvent::Resized(physical_size) => {
                            tracing::debug!(?physical_size, "window resize");
                            if physical_size.width > 0 && physical_size.height > 0 {
                                window.config.width = physical_size.width;
                                window.config.height = physical_size.height;
                                window
                                    .surface
                                    .configure(&window.render_context.device, &window.config);
                                // todo: update SceneView (camera)
                            }
                        }
                        winit::event::WindowEvent::RedrawRequested => {
                            if let Some(scene_view) = &window.scene_view {
                                let texture = window
                                    .surface
                                    .get_current_texture()
                                    .expect("get_current_texture failed");
                                render_scene(
                                    scene_view,
                                    &texture.texture,
                                    &window.render_context.device,
                                    &window.render_context.queue,
                                    &window.pipeline,
                                    &window.render_buffer,
                                );
                                window.window.pre_present_notify();
                                texture.present();
                            }
                        }
                        _ => {}
                    }
                    //(window_state.event_handler)(&self.handle, event);
                }
            }
            Event::UserEvent(command) => {
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
                        scene_view,
                        tx_events,
                        render_context,
                    } => {
                        let render_context = render_context
                            .map(Arc::new)
                            .or_else(|| self.shared_context.clone())
                            .expect("missing render context");
                        self.initialize_window(
                            window,
                            surface,
                            initial_size,
                            scene_view,
                            tx_events,
                            render_context,
                        );
                    }
                    Command::DestroyWindow { window_id } => {
                        self.destroy_window(window_id);
                    }
                    Command::SetSceneView {
                        window_id,
                        scene_view,
                    } => {
                        if let Some(window) = self.windows.get_mut(&window_id) {
                            window.scene_view = scene_view;
                        }
                    }
                    Command::CreateTexture { image: _ } => {
                        // todo
                    }
                }
            }
            _ => {}
        }
    }

    fn create_window(
        &self,
        canvas: HtmlCanvasElement,
        instance: Arc<Instance>,
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

    fn initialize_window(
        &mut self,
        window: Arc<Window>,
        surface: Surface<'static>,
        initial_size: PhysicalSize<u32>,
        scene_view: Option<SceneView>,
        tx_events: mpsc::Sender<window::Event>,
        render_context: Arc<RenderContext>,
    ) {
        let window_id = window.id();

        let surface_caps = surface.get_capabilities(&render_context.adapter);
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
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: initial_size.width,
            height: initial_size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&render_context.device, &config);

        // create render pipeline

        let shader = render_context
            .device
            .create_shader_module(wgpu::include_wgsl!("shader/shader.wgsl"));

        let pipeline_layout =
            render_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let pipeline =
            render_context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[Vertex::layout()],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        const VERTICES: &[Vertex] = &[
            Vertex {
                position: [-0.0868241, 0.49240386, 0.0],
                color: [0.5, 0.0, 0.5],
            }, // A
            Vertex {
                position: [-0.49513406, 0.06958647, 0.0],
                color: [0.5, 0.0, 0.5],
            }, // B
            Vertex {
                position: [-0.21918549, -0.44939706, 0.0],
                color: [0.5, 0.0, 0.5],
            }, // C
            Vertex {
                position: [0.35966998, -0.3473291, 0.0],
                color: [0.5, 0.0, 0.5],
            }, // D
            Vertex {
                position: [0.44147372, 0.2347359, 0.0],
                color: [0.5, 0.0, 0.5],
            }, // E
        ];
        let vertex_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
        let index_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let render_buffer = RenderBuffer {
            vertex_buffer,
            index_buffer,
            num_indices: INDICES.len(),
        };

        // finish

        self.windows.insert(
            window_id,
            WindowState {
                window,
                surface,
                config,
                scene_view,
                pipeline,
                render_buffer,
                tx_events,
                render_context,
            },
        );
    }

    fn destroy_window(&mut self, window_id: WindowId) {
        if let Some(_window) = self.windows.remove(&window_id) {
            // we think we can just drop everything
        }
    }
}

struct WindowState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    scene_view: Option<SceneView>,
    pipeline: wgpu::RenderPipeline,
    render_buffer: RenderBuffer,
    tx_events: mpsc::Sender<window::Event>,
    render_context: Arc<RenderContext>,
}

#[derive(Clone)]
pub struct SceneView {
    pub scene: Scene,
    pub camera: Entity,
}

struct RenderBuffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: usize,
}

#[derive(Clone, Copy, Debug, Zeroable, Pod)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

fn render_scene(
    scene_view: &SceneView,
    target_texture: &wgpu::Texture,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::RenderPipeline,
    render_buffer: &RenderBuffer,
) {
    let target_view = target_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("scene render encoder"),
    });

    let mut world = scene_view.scene.world.lock().unwrap();

    let (camera_transform, clear_color) = {
        let (transform, camera) = world
            .query_one_mut::<(&Transform, &Camera)>(scene_view.camera)
            .unwrap();
        (
            camera.projection * transform.transform.inverse(),
            camera
                .clear_color
                .map(|c| convert_color_palette_to_wgpu(c.into_format())),
        )
    };

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("scene render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: clear_color
                        .map(|c| wgpu::LoadOp::Clear(c))
                        .unwrap_or(wgpu::LoadOp::Load),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(pipeline);
        render_pass.set_vertex_buffer(0, render_buffer.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            render_buffer.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..render_buffer.num_indices as u32, 0, 0..1);
    }

    for (_entity, (transform, _mesh)) in world.query_mut::<(&Transform, &Mesh)>() {
        let _transform = &camera_transform * &transform.transform;

        // todo
    }

    queue.submit(Some(encoder.finish()));
}

fn convert_color_palette_to_wgpu(color: Srgba<f64>) -> wgpu::Color {
    wgpu::Color {
        r: color.red,
        g: color.green,
        b: color.blue,
        a: color.alpha,
    }
}

fn canvas_size(canvas: &HtmlCanvasElement) -> PhysicalSize<u32> {
    let width = std::cmp::max(1, canvas.width());
    let height = std::cmp::max(1, canvas.height());
    PhysicalSize::new(width, height)
}
