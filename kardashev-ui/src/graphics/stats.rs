use std::{
    ops::{
        Deref,
        DerefMut,
    },
    sync::OnceLock,
};

use parking_lot::RwLock;
use wgpu::util::DeviceExt;

#[derive(Clone, Debug, Default)]
pub struct ResourceUsages {
    pub buffers: usize,
    pub textures: usize,
    pub texture_views: usize,
    pub bind_groups: usize,
    pub bind_group_layouts: usize,
    pub render_pipelines: usize,
    pub pipeline_layouts: usize,
    pub samplers: usize,
    pub command_encoders: usize,
    pub shader_modules: usize,
    pub buffer_memory: u64,
    pub texture_memory: u64,
}

impl ResourceUsages {
    pub fn get() -> Self {
        (*get_stats().read()).clone()
    }
}

pub trait Tracked {
    type Data;

    fn begin(&self, data: &Self::Data, stats: &mut ResourceUsages);
    fn end(&self, data: &Self::Data, stats: &mut ResourceUsages);
}

#[derive(Clone, Debug)]
pub struct Track<T: Tracked> {
    inner: Option<T>,
    data: T::Data,
}

impl<T: Tracked> Track<T> {
    pub fn new(inner: T, data: T::Data) -> Self {
        let mut stats = get_stats().write();
        T::begin(&inner, &data, &mut stats);
        Self {
            inner: Some(inner),
            data,
        }
    }

    pub fn into_inner(mut self) -> T {
        let inner = self.inner.take().unwrap();
        let mut stats = get_stats().write();
        T::end(&inner, &self.data, &mut stats);
        inner
    }
}

impl<T: Tracked> Drop for Track<T> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            let mut stats = get_stats().write();
            T::end(&inner, &self.data, &mut stats);
        }
    }
}

impl<T: Tracked> Deref for Track<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl<T: Tracked> DerefMut for Track<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

fn get_stats() -> &'static RwLock<ResourceUsages> {
    static STATS: OnceLock<RwLock<ResourceUsages>> = OnceLock::new();
    STATS.get_or_init(|| RwLock::new(ResourceUsages::default()))
}

pub fn track<T: Tracked<Data = ()>>(value: T) -> Track<T> {
    Track::new(value, ())
}

pub fn track_with<T: Tracked>(value: T, data: T::Data) -> Track<T> {
    Track::new(value, data)
}

impl Tracked for wgpu::Buffer {
    type Data = ();
    fn begin(&self, _data: &Self::Data, stats: &mut ResourceUsages) {
        stats.buffers += 1;
        stats.buffer_memory += self.size();
    }

    fn end(&self, _data: &Self::Data, stats: &mut ResourceUsages) {
        stats.buffers -= 1;
        stats.buffer_memory -= self.size();
    }
}

impl Tracked for wgpu::Texture {
    type Data = u64;

    fn begin(&self, data: &Self::Data, stats: &mut ResourceUsages) {
        stats.textures += 1;
        stats.texture_memory += *data;
    }

    fn end(&self, data: &Self::Data, stats: &mut ResourceUsages) {
        stats.textures -= 1;
        stats.texture_memory -= *data;
    }
}

macro_rules! simple_counter {
    {$(
        $ty:ty => $field:ident,
    )*} => {
        $(
            impl Tracked for $ty {
                type Data = ();

                fn begin(&self, _data: &Self::Data, stats: &mut ResourceUsages) {
                    stats.$field += 1;
                }

                fn end(&self, _data: &Self::Data, stats: &mut ResourceUsages) {
                    stats.$field -= 1;
                }
            }
        )*
    };
}

simple_counter! {
    wgpu::TextureView => texture_views,
    wgpu::BindGroup => bind_groups,
    wgpu::BindGroupLayout => bind_group_layouts,
    wgpu::RenderPipeline => render_pipelines,
    wgpu::PipelineLayout => pipeline_layouts,
    wgpu::Sampler => samplers,
    wgpu::CommandEncoder => command_encoders,
    wgpu::ShaderModule => shader_modules,
}

/// Wrapper around [`wgpu::Device`] that tracks usage of resources
#[derive(Debug)]
pub struct TrackedDevice {
    inner: wgpu::Device,
}

impl TrackedDevice {
    pub fn new(inner: wgpu::Device) -> Self {
        Self { inner }
    }
}

impl Deref for TrackedDevice {
    type Target = wgpu::Device;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TrackedDevice {
    pub fn create_buffer(&self, desc: &wgpu::BufferDescriptor) -> Track<wgpu::Buffer> {
        track(self.inner.create_buffer(desc))
    }

    pub fn create_texture(&self, desc: &wgpu::TextureDescriptor) -> Track<wgpu::Texture> {
        // fixme: how to we more accurately determine the size of a texture in bytes?
        let size = u64::from(desc.size.width)
            * u64::from(desc.size.height)
            * u64::from(desc.size.depth_or_array_layers)
            * u64::from(desc.format.target_pixel_byte_cost().unwrap_or(4));
        track_with(self.inner.create_texture(desc), size)
    }

    pub fn create_texture_with_data(
        &self,
        queue: &wgpu::Queue,
        desc: &wgpu::TextureDescriptor,
        order: wgpu::util::TextureDataOrder,
        data: &[u8],
    ) -> Track<wgpu::Texture> {
        track_with(
            self.inner
                .create_texture_with_data(queue, desc, order, data),
            data.len().try_into().unwrap(),
        )
    }
}
