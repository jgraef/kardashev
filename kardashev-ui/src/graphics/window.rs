use std::sync::{
    atomic::{
        AtomicUsize,
        Ordering,
    },
    Arc,
};

use image::RgbaImage;
use tokio::sync::oneshot;
use winit::dpi::PhysicalSize;

use super::{
    texture::Texture,
    Command,
    Error,
    Graphics,
};

pub struct Window {
    graphics: Graphics,
    window: Arc<winit::window::Window>,
    reference_count: Arc<AtomicUsize>,
}

impl Window {
    pub(super) fn new(graphics: Graphics, window: Arc<winit::window::Window>) -> Self {
        Self {
            graphics,
            window,
            reference_count: Arc::new(AtomicUsize::new(1)),
        }
    }

    pub async fn load_texture(
        &self,
        image: RgbaImage,
        label: Option<String>,
    ) -> Result<Texture, Error> {
        let (tx_response, rx_response) = oneshot::channel();
        self.graphics.send_command(Command::LoadTexture {
            window_id: self.window.id(),
            image,
            label,
            tx_response,
        });
        rx_response.await.expect("tx_response dropped")
    }
}

impl Clone for Window {
    fn clone(&self) -> Self {
        self.reference_count.fetch_add(1, Ordering::Relaxed);
        Self {
            graphics: self.graphics.clone(),
            window: self.window.clone(),
            reference_count: self.reference_count.clone(),
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if self.reference_count.fetch_sub(1, Ordering::Relaxed) <= 1 {
            self.graphics.destroy_window(self.window.id());
        }
    }
}

pub trait WindowHandler: 'static {
    fn on_resize(&mut self, new_size: PhysicalSize<u32>);
}

pub struct NullEventHandler;

impl WindowHandler for NullEventHandler {
    fn on_resize(&mut self, _new_size: PhysicalSize<u32>) {}
}
