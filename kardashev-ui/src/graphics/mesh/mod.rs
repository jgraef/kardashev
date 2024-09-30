//pub mod teapot;
pub mod shape;

use std::{
    hash::Hash,
    sync::Arc,
};

use kardashev_protocol::assets::AssetId;
use linear_map::LinearMap;
use wgpu::util::DeviceExt;

use super::{
    rendering_system::LoadContext,
    BackendId,
};
use crate::{
    graphics::rendering_system::Vertex,
    utils::thread_local_cell::ThreadLocalCell,
};

#[derive(Debug)]
pub struct Mesh {
    asset_id: Option<AssetId>,
    data: Option<MeshData>,
    loaded: LinearMap<BackendId, LoadedMesh>,
}

impl From<MeshData> for Mesh {
    fn from(mesh_data: MeshData) -> Self {
        Mesh {
            asset_id: None,
            data: Some(mesh_data),
            loaded: LinearMap::new(),
        }
    }
}

impl Mesh {
    pub(super) fn loaded(&mut self, context: &LoadContext) -> Option<&LoadedMesh> {
        match self.loaded.entry(context.backend.id()) {
            linear_map::Entry::Occupied(occupied) => Some(occupied.into_mut()),
            linear_map::Entry::Vacant(vacant) => {
                let mesh_data = self.data.as_ref()?;
                let loaded = mesh_data.load(context);
                Some(vacant.insert(loaded))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct MeshData {
    pub primitive_topology: PrimitiveTopology,
    pub indices: Vec<u16>,
    pub vertices: Vec<Vertex>,
}

impl MeshData {
    fn load(&self, context: &LoadContext) -> LoadedMesh {
        let vertex_buffer =
            context
                .backend
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&self.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            context
                .backend
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&self.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        LoadedMesh {
            id: LoadedMeshId(index_buffer.global_id()),
            buffers: Arc::new(ThreadLocalCell::new(LoadedMeshBuffers {
                vertex_buffer,
                index_buffer,
                num_indices: self.indices.len() as u32,
            })),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
}

#[derive(Clone, Debug)]
pub struct LoadedMesh {
    id: LoadedMeshId,
    buffers: Arc<ThreadLocalCell<LoadedMeshBuffers>>,
}

impl LoadedMesh {
    pub fn id(&self) -> LoadedMeshId {
        self.id
    }

    pub fn buffers(&self) -> &LoadedMeshBuffers {
        self.buffers.get()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoadedMeshId(wgpu::Id<wgpu::Buffer>);

#[derive(Debug)]
pub struct LoadedMeshBuffers {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

pub trait Meshable {
    type Output: MeshBuilder;

    fn mesh(&self) -> Self::Output;
}

pub trait MeshBuilder {
    fn build(&self) -> MeshData;
}
