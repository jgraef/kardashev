use std::sync::{
    atomic::{
        AtomicUsize,
        Ordering,
    },
    Arc,
};

use winit::dpi::PhysicalSize;

use super::Graphics;

pub struct Window {
    renderer: Graphics,
    window: Arc<winit::window::Window>,
    reference_count: Arc<AtomicUsize>,
}

impl Window {
    pub(super) fn new(renderer: Graphics, window: Arc<winit::window::Window>) -> Self {
        Self {
            renderer,
            window,
            reference_count: Arc::new(AtomicUsize::new(1)),
        }
    }
}

impl Clone for Window {
    fn clone(&self) -> Self {
        self.reference_count.fetch_add(1, Ordering::Relaxed);
        Self {
            renderer: self.renderer.clone(),
            window: self.window.clone(),
            reference_count: self.reference_count.clone(),
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if self.reference_count.fetch_sub(1, Ordering::Relaxed) <= 1 {
            self.renderer.destroy_window(self.window.id());
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
