use std::sync::Arc;

use kardashev_protocol::assets::{
    AssetId,
    Vertex,
};
use palette::Srgba;

use crate::{
    graphics::backend::BackendId,
    utils::any_cache::AnyArcCache,
};

pub fn wgpu_buffer_size<T>() -> u64 {
    let unpadded_size: u64 = std::mem::size_of::<T>()
        .try_into()
        .expect("failed to convert usize to u64");
    let align_mask = wgpu::COPY_BUFFER_ALIGNMENT - 1;
    let padded_size = ((unpadded_size + align_mask) & !align_mask).max(wgpu::COPY_BUFFER_ALIGNMENT);
    padded_size
}

pub fn color_to_wgpu(color: Srgba<f64>) -> wgpu::Color {
    wgpu::Color {
        r: color.red,
        g: color.green,
        b: color.blue,
        a: color.alpha,
    }
}

pub fn color_to_array<T: Copy>(color: Srgba<T>) -> [T; 4] {
    [color.red, color.green, color.blue, color.alpha]
}

pub trait HasVertexBufferLayout {
    fn layout() -> wgpu::VertexBufferLayout<'static>;
}

impl HasVertexBufferLayout for Vertex {
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
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Debug, Default)]
pub struct GpuResourceCache {
    inner: AnyArcCache<(BackendId, AssetId)>,
}

impl GpuResourceCache {
    pub fn get<T>(&self, backend_id: BackendId, asset_id: AssetId) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.inner.get((backend_id, asset_id))
    }

    pub fn insert<T>(&mut self, backend_id: BackendId, asset_id: AssetId, value: &Arc<T>)
    where
        T: Send + Sync + 'static,
    {
        self.inner.insert((backend_id, asset_id), value)
    }

    pub fn get_or_try_insert<T, F, E>(
        &mut self,
        backend_id: BackendId,
        asset_id: AssetId,
        insert: F,
    ) -> Result<Arc<T>, E>
    where
        T: Send + Sync + 'static,
        F: FnOnce() -> Result<Arc<T>, E>,
    {
        self.inner.get_or_try_insert((backend_id, asset_id), insert)
    }

    pub fn get_or_insert<T, F>(
        &mut self,
        backend_id: BackendId,
        asset_id: AssetId,
        insert: F,
    ) -> Arc<T>
    where
        T: Send + Sync + 'static,
        F: FnOnce() -> Arc<T>,
    {
        self.inner.get_or_insert((backend_id, asset_id), insert)
    }
}
