use std::{
    any::type_name,
    marker::PhantomData,
    ops::RangeBounds,
    sync::Arc,
};

use bytemuck::Pod;
use kardashev_protocol::assets::{
    AssetId,
    TextureFormat,
    Vertex,
};
use nalgebra::Vector3;
use palette::{
    Srgb,
    Srgba,
};

use crate::{
    ecs::resource::Resources,
    graphics::{
        backend::{
            Backend,
            BackendId,
        },
        draw_batch::DrawBatcher,
        material::{
            Material,
            PipelineMaterial,
        },
        mesh::Mesh,
        render_3d::{
            MeshMaterialPair,
            MeshMaterialPairKey,
        },
        transform::GlobalTransform,
        SurfaceSize,
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

pub trait Srgba64Ext {
    fn as_wgpu(&self) -> wgpu::Color;
}

impl Srgba64Ext for Srgba<f64> {
    fn as_wgpu(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.red,
            g: self.green,
            b: self.blue,
            a: self.alpha,
        }
    }
}

pub trait Srgba32Ext {
    fn as_array4(&self) -> [f32; 4];
}

impl Srgba32Ext for Srgba<f32> {
    fn as_array4(&self) -> [f32; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }
}

pub trait Srgb32Ext {
    fn as_array3(&self) -> [f32; 3];
    fn as_array4(&self) -> [f32; 4];
}

impl Srgb32Ext for Srgb<f32> {
    fn as_array3(&self) -> [f32; 3] {
        [self.red, self.green, self.blue]
    }

    fn as_array4(&self) -> [f32; 4] {
        [self.red, self.green, self.blue, 1.0]
    }
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
                // @location(0) position: vec3<f32>,
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // @location(1) tex_coords: vec2<f32>,
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // @location(2) normal: vec3<f32>,
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // @location(3) tangent: vec3<f32>,
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // @location(4) bitangent: vec3<f32>,
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
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
    pub fn push_view(&mut self) {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        });
    }

    pub fn push_sampler(&mut self) {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
    }

    pub fn push_view_and_sampler(&mut self) {
        self.push_view();
        self.push_sampler();
    }

    pub fn build(&self, device: &wgpu::Device, label: Option<&str>) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label,
            entries: &self.entries,
        })
    }
}

pub trait TextureFormatExt {
    fn as_wgpu(&self) -> wgpu::TextureFormat;
}

impl TextureFormatExt for TextureFormat {
    fn as_wgpu(&self) -> wgpu::TextureFormat {
        match self {
            TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        }
    }
}

/// A general-purpose uniform buffer
#[derive(Debug)]
pub struct UniformBuffer<T> {
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    _ty: PhantomData<T>,
}

impl<T> UniformBuffer<T> {
    pub fn new(backend: &Backend) -> Self {
        let type_name = type_name::<T>();

        let buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{type_name} buffer")),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            size: wgpu_buffer_size::<T>(),
        });

        let bind_group_layout =
            backend
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(&format!("{type_name} bind group layout")),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let bind_group = backend
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("{type_name} bind group")),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        Self {
            buffer,
            bind_group_layout,
            bind_group,
            _ty: PhantomData,
        }
    }
}

impl<T: Pod> UniformBuffer<T> {
    pub fn write(&self, backend: &Backend, data: &T) {
        backend
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::bytes_of(data));
    }
}

/// A general-purpose texture buffer
#[derive(Debug)]
pub struct TextureBuffer {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub size: SurfaceSize,
    pub label: Option<String>,
}

impl TextureBuffer {
    pub fn new(
        backend: &Backend,
        size: SurfaceSize,
        format: wgpu::TextureFormat,
        label: Option<&str>,
    ) -> Self {
        let (texture, texture_view) = Self::create_texture(backend, size, format, label);

        Self {
            texture,
            texture_view,
            format,
            size,
            label: label.map(ToOwned::to_owned),
        }
    }

    pub fn resize(&mut self, backend: &Backend, size: SurfaceSize) {
        if self.size != size {
            tracing::debug!(label = ?self.label, ?size, "resizing texture buffer");
            let (texture, texture_view) =
                Self::create_texture(backend, size, self.format, self.label.as_deref());
            self.texture = texture;
            self.texture_view = texture_view;
            self.size = size;
        }
    }

    fn create_texture(
        backend: &Backend,
        surface_size: SurfaceSize,
        format: wgpu::TextureFormat,
        label: Option<&str>,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = backend.device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: surface_size.width,
                height: surface_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, texture_view)
    }
}

pub fn batch_meshes_with_material<M: PipelineMaterial, I: Pod>(
    world: &hecs::World,
    resources: &mut Resources,
    backend: &Backend,
    draw_batcher: &mut DrawBatcher<MeshMaterialPairKey, MeshMaterialPair<M>, I>,
    material_bind_group_layout: &wgpu::BindGroupLayout,
    make_instance: impl Fn(&GlobalTransform, &M) -> I,
) {
    tracing::trace!("batching");

    let mut render_entities = world.query::<(&GlobalTransform, &mut Mesh, &mut Material<M>)>();

    let gpu_resource_cache = resources.get_mut_or_insert_default::<GpuResourceCache>();

    for (_entity, (transform, mesh, material)) in render_entities.iter() {
        // todo: handle errors

        let instance = make_instance(transform, &material.cpu);

        let Ok(mesh_gpu) = mesh.gpu(backend, gpu_resource_cache)
        else {
            continue;
        };

        let Ok(material_gpu) =
            material.gpu(backend, gpu_resource_cache, material_bind_group_layout)
        else {
            continue;
        };

        draw_batcher.push(
            MeshMaterialPairKey {
                mesh: mesh_gpu.get().id(),
                material: material_gpu.get().id(),
            },
            || {
                MeshMaterialPair {
                    mesh: mesh_gpu.clone(),
                    material: material_gpu.clone(),
                }
            },
            instance,
        );
    }
}

pub fn draw_batched_meshes_with_materials<M: PipelineMaterial, I: Pod>(
    render_pass: &mut wgpu::RenderPass,
    backend: &Backend,
    draw_batcher: &mut DrawBatcher<MeshMaterialPairKey, MeshMaterialPair<M>, I>,
    instance_buffer_slot: u32,
    vertex_buffer_slot: u32,
    material_bind_group_index: u32,
) {
    if let Some(prepared_batch) = draw_batcher.prepare(backend) {
        render_pass.set_vertex_buffer(instance_buffer_slot, prepared_batch.instance_buffer);

        for batch_item in prepared_batch {
            let mesh = batch_item.value.mesh.get();
            let material = batch_item.value.material.get();

            render_pass.set_vertex_buffer(vertex_buffer_slot, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(material_bind_group_index, &material.bind_group, &[]);
            render_pass.draw_indexed(0..mesh.num_indices as u32, 0, batch_item.range);
        }
    }
}
