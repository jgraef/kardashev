use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Texture {
    pub(super) inner: Arc<wgpu::Texture>,
}

impl Texture {
    pub fn view(&self) -> TextureView {
        TextureView {
            inner: Arc::new(self.inner.create_view(&Default::default())),
        }
    }
}

pub struct TextureView {
    pub(super) inner: Arc<wgpu::TextureView>,
}
