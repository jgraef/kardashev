use std::{
    collections::HashMap,
    ops::Range,
    sync::Arc,
};

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
        render_3d::Instance,
        utils::InstanceBuffer,
    },
    utils::thread_local_cell::ThreadLocalCell,
};

#[derive(Debug)]
pub struct DrawBatcher {
    instance_buffer: InstanceBuffer<Instance>,
    entries: HashMap<DrawBatchKey, DrawBatchEntry>,
    reuse_instance_vecs: Vec<Vec<Instance>>,
    batches: Vec<DrawBatch>,
}

impl DrawBatcher {
    const INITIAL_BUFFER_SIZE: usize = 1024;

    pub fn new(backend: &Backend) -> Self {
        Self {
            instance_buffer: InstanceBuffer::new(backend, Self::INITIAL_BUFFER_SIZE),
            entries: HashMap::with_capacity(Self::INITIAL_BUFFER_SIZE),
            reuse_instance_vecs: vec![],
            batches: vec![],
        }
    }

    pub fn draw(&mut self, backend: &Backend, render_pass: &mut wgpu::RenderPass) {
        // create instance list
        for (_, mut entry) in self.entries.drain() {
            let start_index = self.instance_buffer.len() as u32;
            self.instance_buffer.extend(entry.instances.drain(..));
            let end_index = self.instance_buffer.len() as u32;

            self.batches.push(DrawBatch {
                range: start_index..end_index,
                mesh: entry.mesh,
                material: entry.material,
            });

            self.reuse_instance_vecs.push(entry.instances);
        }

        tracing::trace!(
            num_instances = self.instance_buffer.len(),
            num_batches = self.batches.len(),
            "drawing batched"
        );

        if self.batches.len() > 0 {
            // write instance data to gpu
            self.instance_buffer.upload_and_clear(backend);

            // render batches
            for batch in self.batches.drain(..) {
                tracing::trace!(?batch.range, "drawing batch");

                let mesh = batch.mesh.get();
                let material = batch.material.get();

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.set_bind_group(0, &material.bind_group, &[]);
                render_pass.draw_indexed(0..mesh.num_indices as u32, 0, batch.range);
            }
        }
    }

    pub fn push(
        &mut self,
        mesh: &Arc<ThreadLocalCell<GpuMesh>>,
        material: &Arc<ThreadLocalCell<GpuMaterial>>,
        instance: Instance,
    ) {
        self.entries
            .entry(DrawBatchKey {
                mesh_id: mesh.get().id(),
                material_id: material.get().id(),
            })
            .or_insert_with(|| {
                DrawBatchEntry {
                    instances: self.reuse_instance_vecs.pop().unwrap_or_default(),
                    mesh: mesh.clone(),
                    material: material.clone(),
                }
            })
            .instances
            .push(instance);
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct DrawBatchKey {
    mesh_id: GpuMeshId,
    material_id: GpuMaterialId,
}

#[derive(Debug)]
struct DrawBatchEntry {
    instances: Vec<Instance>,
    mesh: Arc<ThreadLocalCell<GpuMesh>>,
    material: Arc<ThreadLocalCell<GpuMaterial>>,
}

#[derive(Debug)]
struct DrawBatch {
    range: Range<u32>,
    mesh: Arc<ThreadLocalCell<GpuMesh>>,
    material: Arc<ThreadLocalCell<GpuMaterial>>,
}

fn create_instance_buffer(backend: &Backend, size: usize) -> wgpu::Buffer {
    tracing::debug!(size, "allocating instance buffer");

    backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("instance buffer"),
        size: (size * std::mem::size_of::<Instance>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
