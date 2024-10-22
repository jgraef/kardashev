use std::{
    marker::PhantomData,
    ops::RangeBounds,
    sync::Arc,
};

use bytemuck::Pod;
use kardashev_protocol::assets::{
    AssetId,
    Vertex,
};
use nalgebra::Vector3;
use palette::{
    Srgb,
    Srgba,
};

use crate::{
    graphics::backend::{
        Backend,
        BackendId,
    },
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

pub fn srgba_to_wgpu(color: Srgba<f64>) -> wgpu::Color {
    wgpu::Color {
        r: color.red,
        g: color.green,
        b: color.blue,
        a: color.alpha,
    }
}

pub fn srgba_to_array4<T: Copy>(color: Srgba<T>) -> [T; 4] {
    [color.red, color.green, color.blue, color.alpha]
}

pub fn srgb_to_array4<T: Copy + Default>(color: Srgb<T>) -> [T; 4] {
    [color.red, color.green, color.blue, Default::default()]
}

pub fn vector3_to_array4<T: Copy + Default>(vector: Vector3<T>) -> [T; 4] {
    let mut array: [T; 4] = Default::default();
    array[..3].copy_from_slice(vector.as_slice());
    array
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

#[derive(Debug)]
pub struct ResizableVertexBuffer<T> {
    buffer: wgpu::Buffer,
    capacity: usize,
    _instance_type: PhantomData<T>,
}

impl<T> ResizableVertexBuffer<T> {
    pub fn new(backend: &Backend, initial_capacity: usize) -> Self {
        let buffer = Self::create_instance_buffer(backend, initial_capacity);
        Self {
            buffer,
            capacity: initial_capacity,
            _instance_type: PhantomData,
        }
    }

    /// Allocates a new buffer such that it can hold `capacity` elements.
    ///
    /// If `capacity` is not greater than the current buffer's capacity, this
    /// does nothing.
    ///
    /// This does **not** copy the contents to the new buffer.
    ///
    /// You can also just call [`Self::write`] with your data, and it'll grow
    /// the buffer as necessary.
    pub fn grow(&mut self, backend: &Backend, capacity: usize) {
        if capacity > self.capacity {
            let capacity = capacity.max(self.capacity * 2);
            self.buffer = Self::create_instance_buffer(backend, capacity);
            self.capacity = capacity;
        }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn slice(&self, bounds: impl RangeBounds<wgpu::BufferAddress>) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(bounds)
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    fn create_instance_buffer(backend: &Backend, capacity: usize) -> wgpu::Buffer {
        tracing::trace!(capacity, "allocating instance buffer");

        backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance buffer"),
            size: (capacity * std::mem::size_of::<T>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}

impl<T: Pod> ResizableVertexBuffer<T> {
    pub fn write(&mut self, backend: &Backend, data: &[T]) {
        self.grow(backend, data.len());
        backend
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
    }
}

/// A [`ResizableVertexBuffer`] with a buffer (in host memory) for staging -
/// usually used for sending instances to the GPU.
#[derive(Debug)]
pub struct InstanceBuffer<T> {
    buffer: ResizableVertexBuffer<T>,
    staging: Vec<T>,
}

impl<T> InstanceBuffer<T> {
    pub fn new(backend: &Backend, initial_capacity: usize) -> Self {
        Self {
            buffer: ResizableVertexBuffer::new(backend, initial_capacity),
            staging: Vec::with_capacity(initial_capacity),
        }
    }

    pub fn clear(&mut self) {
        self.staging.clear();
    }

    pub fn push(&mut self, instance: T) {
        self.staging.push(instance);
    }

    pub fn extend(&mut self, instances: impl IntoIterator<Item = T>) {
        self.staging.extend(instances);
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        self.buffer.buffer()
    }

    pub fn slice(&self, bounds: impl RangeBounds<wgpu::BufferAddress>) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(bounds)
    }

    pub fn len(&self) -> usize {
        self.staging.len()
    }

    pub fn is_empty(&self) -> bool {
        self.staging.is_empty()
    }
}

impl<T: Pod> InstanceBuffer<T> {
    pub fn upload(&mut self, backend: &Backend) {
        self.buffer.write(backend, &self.staging);
    }

    pub fn upload_and_clear(&mut self, backend: &Backend) {
        self.upload(backend);
        self.staging.clear();
    }
}

#[derive(Clone, Debug, Default)]
pub struct MaterialBindGroupLayoutBuilder {
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl MaterialBindGroupLayoutBuilder {
    pub fn push_view_and_sampler(&mut self) {
        let index = self.entries.len() as u32;

        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: index,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        });

        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: index + 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            // This should match the filterable field of the
            // corresponding Texture entry above.
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
    }

    pub fn build(&self, device: &wgpu::Device, label: Option<&str>) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label,
            entries: &self.entries,
        })
    }
}
