use std::{
    collections::HashMap,
    hash::Hash,
    ops::Range,
};

use bytemuck::Pod;

use crate::graphics::{
    backend::Backend,
    utils::InstanceBuffer,
};

#[derive(Debug)]
pub struct DrawBatcher<K, V, I> {
    instance_buffer: InstanceBuffer<I>,
    entries: HashMap<K, BatchEntry<V, I>>,
    reuse_instance_vecs: Vec<Vec<I>>,
    items: Vec<BatchItem<V>>,
}

impl<K, V, I> DrawBatcher<K, V, I> {
    const INITIAL_BUFFER_SIZE: usize = 1024;

    pub fn new(backend: &Backend) -> Self {
        Self {
            instance_buffer: InstanceBuffer::new(backend, Self::INITIAL_BUFFER_SIZE),
            entries: HashMap::with_capacity(Self::INITIAL_BUFFER_SIZE),
            reuse_instance_vecs: vec![],
            items: vec![],
        }
    }
}

impl<K: Eq + Hash, V, I> DrawBatcher<K, V, I> {
    pub fn push(&mut self, key: K, value: impl FnOnce() -> V, instance: I) {
        self.entries
            .entry(key)
            .or_insert_with(|| {
                BatchEntry {
                    instances: self.reuse_instance_vecs.pop().unwrap_or_default(),
                    value: value(),
                }
            })
            .instances
            .push(instance);
    }
}

impl<K, V, I: Pod> DrawBatcher<K, V, I> {
    pub fn prepare(&mut self, backend: &Backend) -> Option<PreparedBatch<V>> {
        // create instance list
        for (_, mut entry) in self.entries.drain() {
            let start_index = self.instance_buffer.len() as u32;
            self.instance_buffer.extend(entry.instances.drain(..));
            let end_index = self.instance_buffer.len() as u32;

            self.items.push(BatchItem {
                range: start_index..end_index,
                value: entry.value,
            });

            self.reuse_instance_vecs.push(entry.instances);
        }

        tracing::trace!(
            num_instances = self.instance_buffer.len(),
            num_batches = self.items.len(),
            "finished batch"
        );

        if self.items.len() > 0 {
            self.instance_buffer.upload_and_clear(backend);

            Some(PreparedBatch {
                instance_buffer: self.instance_buffer.slice(..),
                batch_items: self.items.drain(..),
            })
        }
        else {
            None
        }
    }
}

#[derive(Debug)]
struct BatchEntry<V, I> {
    value: V,
    instances: Vec<I>,
}

#[derive(Debug)]
pub struct BatchItem<V> {
    pub range: Range<u32>,
    pub value: V,
}
#[derive(Debug)]
pub struct PreparedBatch<'a, V> {
    pub instance_buffer: wgpu::BufferSlice<'a>,
    batch_items: std::vec::Drain<'a, BatchItem<V>>,
}

impl<'a, V> Iterator for PreparedBatch<'a, V> {
    type Item = BatchItem<V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.batch_items.next()
    }
}

fn create_instance_buffer<I>(backend: &Backend, size: usize) -> wgpu::Buffer {
    tracing::debug!(size, "allocating instance buffer");

    backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("instance buffer"),
        size: (size * std::mem::size_of::<I>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
