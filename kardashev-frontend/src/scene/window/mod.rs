use std::{
    pin::Pin,
    sync::{
        atomic::{
            AtomicUsize,
            Ordering,
        },
        Arc,
    },
    task::{
        Context,
        Poll,
    },
};

use futures::Stream;
use tokio::sync::mpsc;

use super::renderer::SceneRenderer;

pub mod leptos;

#[derive(Clone, Debug)]
pub enum Event {
    // todo
}

pub struct Window {
    scene_renderer: SceneRenderer,
    window: Arc<winit::window::Window>,
    reference_count: Arc<AtomicUsize>,
}

impl Window {
    pub(super) fn new(scene_renderer: SceneRenderer, window: Arc<winit::window::Window>) -> Self {
        Self {
            scene_renderer,
            window,
            reference_count: Arc::new(AtomicUsize::new(1)),
        }
    }
}

impl Clone for Window {
    fn clone(&self) -> Self {
        self.reference_count.fetch_add(1, Ordering::Relaxed);
        Self {
            scene_renderer: self.scene_renderer.clone(),
            window: self.window.clone(),
            reference_count: self.reference_count.clone(),
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if self.reference_count.fetch_sub(1, Ordering::Relaxed) <= 1 {
            self.scene_renderer.destroy_window(self.window.id());
        }
    }
}

pub struct Events {
    rx: mpsc::Receiver<Event>,
}

impl Events {
    pub(super) fn new(rx: mpsc::Receiver<Event>) -> Self {
        Self { rx }
    }
}

impl Stream for Events {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}
