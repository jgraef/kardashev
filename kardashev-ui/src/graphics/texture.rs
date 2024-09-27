use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Texture {
    pub texture: Arc<wgpu::Texture>,
    pub view: Arc<wgpu::TextureView>,
    pub sampler: Arc<wgpu::Sampler>,
}
