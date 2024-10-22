use std::{
    collections::HashMap,
    ops::Range,
    sync::Arc,
};

use bytemuck::Pod;

use crate::{
    graphics::{
        backend::Backend,
        material::{
            GpuMaterial,
            GpuMaterialId,
        },
        mesh::{
            GpuMesh,
            GpuMeshId,
        },
        utils::InstanceBuffer,
    },
    utils::thread_local_cell::ThreadLocalCell,
};

#[derive(Debug)]
pub struct DrawBatcher<I> {
    instance_buffer: InstanceBuffer<I>,
    entries: HashMap<BatchKey, BatchEntry<I>>,
    reuse_instance_vecs: Vec<Vec<I>>,
    items: Vec<BatchItem>,
}

impl<I> DrawBatcher<I> {
    const INITIAL_BUFFER_SIZE: usize = 1024;

    pub fn new(backend: &Backend) -> Self {
        Self {
            instance_buffer: InstanceBuffer::new(backend, Self::INITIAL_BUFFER_SIZE),
            entries: HashMap::with_capacity(Self::INITIAL_BUFFER_SIZE),
            reuse_instance_vecs: vec![],
            items: vec![],
        }
    }

    pub fn push(
        &mut self,
        mesh: &Arc<ThreadLocalCell<GpuMesh>>,
        material: &Arc<ThreadLocalCell<GpuMaterial>>,
        instance: I,
    ) {
        self.entries
            .entry(BatchKey {
                mesh_id: mesh.get().id(),
                material_id: material.get().id(),
            })
            .or_insert_with(|| {
                BatchEntry {
                    instances: self.reuse_instance_vecs.pop().unwrap_or_default(),
                    mesh: mesh.clone(),
                    material: material.clone(),
                }
            })
            .instances
            .push(instance);
    }
}

impl<I: Pod> DrawBatcher<I> {
    pub fn prepare(&mut self, backend: &Backend) -> Option<PreparedBatch> {
        // create instance list
        for (_, mut entry) in self.entries.drain() {
            let start_index = self.instance_buffer.len() as u32;
            self.instance_buffer.extend(entry.instances.drain(..));
            let end_index = self.instance_buffer.len() as u32;

            self.items.push(BatchItem {
                range: start_index..end_index,
                mesh: entry.mesh,
                material: entry.material,
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

#[derive(Debug, PartialEq, Eq, Hash)]
struct BatchKey {
    mesh_id: GpuMeshId,
    material_id: GpuMaterialId,
}

#[derive(Debug)]
struct BatchEntry<I> {
    instances: Vec<I>,
    mesh: Arc<ThreadLocalCell<GpuMesh>>,
    material: Arc<ThreadLocalCell<GpuMaterial>>,
}

#[derive(Debug)]
pub struct BatchItem {
    pub range: Range<u32>,
    pub mesh: Arc<ThreadLocalCell<GpuMesh>>,
    pub material: Arc<ThreadLocalCell<GpuMaterial>>,
}
#[derive(Debug)]
pub struct PreparedBatch<'a> {
    pub instance_buffer: wgpu::BufferSlice<'a>,
    batch_items: std::vec::Drain<'a, BatchItem>,
}

impl<'a> Iterator for PreparedBatch<'a> {
    type Item = BatchItem;

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
