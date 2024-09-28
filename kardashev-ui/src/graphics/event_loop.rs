use std::{
    collections::HashMap,
    sync::{
        Arc,
        OnceLock,
    },
};

use tokio::sync::{
    mpsc,
    oneshot,
};
use web_sys::HtmlCanvasElement;

// yeah, i know, singletons bad, but whatever...
fn event_loop() -> &'static EventLoop {
    static EVENT_LOOP: OnceLock<EventLoop> = OnceLock::new();
    EVENT_LOOP.get_or_init(|| EventLoop::spawn())
}
// yikes! i bet this is fine lol
// (the problem is that HtmlCanvasElement is not Send or Sync)
unsafe impl Send for EventLoop {}
unsafe impl Sync for EventLoop {}

#[derive(Clone, Debug)]
struct EventLoop {
    proxy: winit::event_loop::EventLoopProxy<Command>,
}

impl EventLoop {
    pub fn spawn() -> Self {
        let event_loop = winit::event_loop::EventLoop::with_user_event()
            .build()
            .expect("failed to create event loop");
        let proxy = event_loop.create_proxy();

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::EventLoopExtWebSys;
            tracing::debug!("spawning window event loop");
            event_loop.spawn_app(App::default());
        }

        Self { proxy }
    }

    fn send_command(&self, command: Command) {
        self.proxy.send_event(command).unwrap();
    }

    pub async fn create_window(&self, canvas: HtmlCanvasElement) -> Window {
        let (tx_result, rx_result) = oneshot::channel();
        let (tx_events, rx_events) = mpsc::channel(16);
        self.send_command(Command::CreateWindow {
            canvas,
            tx_result,
            tx_events,
        });
        let handle = rx_result.await.expect("result sender dropped");
        Window { handle, rx_events }
    }

    pub fn shutdown(&self) {
        self.send_command(Command::Shutdown);
    }
}

#[derive(Debug, Default)]
struct App {
    windows: HashMap<winit::window::WindowId, WindowState>,
}

impl winit::application::ApplicationHandler<Command> for App {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(window) = self.windows.get(&window_id) {
            match window.tx_events.try_send(event) {
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    // `Window` dropped, so we destroy the window
                    self.windows.remove(&window_id);
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    // drop event. what else shall we do?
                    tracing::warn!("receiver full. dropping event");
                }
                Ok(()) => {}
            }
        }
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: Command) {
        match event {
            Command::CreateWindow {
                canvas,
                tx_result,
                tx_events,
            } => {
                #[allow(unused_mut)]
                let mut window_attributes = winit::window::WindowAttributes::default();
                #[allow(unused_variables)]
                let canvas = canvas;

                #[cfg(target_arch = "wasm32")]
                {
                    use winit::platform::web::WindowAttributesExtWebSys;
                    window_attributes = window_attributes.with_canvas(Some(canvas));
                }

                match event_loop.create_window(window_attributes) {
                    Ok(handle) => {
                        let handle = Arc::new(handle);
                        self.windows.insert(
                            handle.id(),
                            WindowState {
                                handle: handle.clone(),
                                tx_events,
                            },
                        );
                        tx_result.send(handle).expect("result receiver dropped");
                    }
                    Err(e) => {
                        tracing::error!(%e, "Failed to create window");
                        // we drop tx_result, so the task waiting for a reply
                        // will panic (they unwrap)
                    }
                }
            }
            Command::Shutdown => {
                event_loop.exit();
            }
        }
    }
}

#[derive(Debug)]
struct WindowState {
    tx_events: mpsc::Sender<winit::event::WindowEvent>,
    handle: Arc<winit::window::Window>,
}

#[derive(Debug)]
enum Command {
    CreateWindow {
        canvas: HtmlCanvasElement,
        tx_result: oneshot::Sender<Arc<winit::window::Window>>,
        tx_events: mpsc::Sender<winit::event::WindowEvent>,
    },
    Shutdown,
}

#[derive(Debug)]
pub struct Window {
    handle: Arc<winit::window::Window>,
    rx_events: mpsc::Receiver<winit::event::WindowEvent>,
}

impl Window {
    pub async fn new(canvas: HtmlCanvasElement) -> Self {
        event_loop().create_window(canvas).await
    }

    pub fn id(&self) -> winit::window::WindowId {
        self.handle.id()
    }

    pub async fn next_event(&mut self) -> winit::event::WindowEvent {
        self.rx_events.recv().await.expect("event sender dropped")
    }
}
