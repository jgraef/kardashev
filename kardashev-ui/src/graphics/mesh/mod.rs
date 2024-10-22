//pub mod teapot;
pub mod shape;

use std::sync::Arc;

use kardashev_client::{
    AssetClient,
    DownloadError,
};
use kardashev_protocol::assets::{
    self as dist,
    AssetId,
};
pub use kardashev_protocol::assets::{
    MeshData as CpuMesh,
    PrimitiveTopology,
    Vertex,
};
use wgpu::util::DeviceExt;

use crate::{
    assets::{
        load::{
            LoadAssetContext,
            LoadFromAsset,
        },
        store::{
            AssetStoreGuard,
            AssetStoreMetaData,
        },
        AssetNotFound,
        MaybeHasAssetId,
    },
    graphics::{
        backend::{
            Backend,
            PerBackend,
        },
        utils::GpuResourceCache,
    },
    utils::{
        thread_local_cell::ThreadLocalCell,
        web_fs::{
            self,
            OpenOptions,
        },
    },
};

#[derive(Clone, Debug)]
pub struct Mesh {
    asset_id: Option<AssetId>,
    label: Option<String>,
    cpu: Option<Arc<CpuMesh>>,
    gpu: PerBackend<Arc<ThreadLocalCell<GpuMesh>>>,
}

impl Mesh {
    pub fn cpu(&self) -> Option<&CpuMesh> {
        self.cpu.as_deref()
    }

    pub fn gpu(
        &mut self,
        backend: &Backend,
        cache: &mut GpuResourceCache,
    ) -> Result<&Arc<ThreadLocalCell<GpuMesh>>, MeshError> {
        self.gpu.get_or_try_insert(backend.id, || {
            let mesh_data = self.cpu.as_ref().ok_or_else(|| MeshError::NoCpuMesh)?;
            if let Some(asset_id) = self.asset_id {
                cache.get_or_try_insert(backend.id, asset_id, || {
                    Ok::<_, MeshError>(Arc::new(ThreadLocalCell::new(load_mesh_to_gpu(
                        mesh_data,
                        self.label.as_deref(),
                        backend,
                    )?)))
                })
            }
            else {
                Ok::<_, MeshError>(Arc::new(ThreadLocalCell::new(load_mesh_to_gpu(
                    &mesh_data,
                    self.label.as_deref(),
                    backend,
                )?)))
            }
        })
    }
}

impl MaybeHasAssetId for Mesh {
    fn maybe_asset_id(&self) -> Option<AssetId> {
        self.asset_id
    }
}

impl LoadFromAsset for Mesh {
    type Dist = dist::Mesh;
    type Error = MeshError;
    type Args = ();

    async fn load<'a, 'b: 'a>(
        asset_id: AssetId,
        _args: (),
        context: &'a mut LoadAssetContext<'b>,
    ) -> Result<Self, MeshError> {
        let dist = context
            .dist_assets
            .get::<dist::Mesh>(asset_id)
            .ok_or_else(|| AssetNotFound { asset_id })?;

        let cpu = context
            .cache
            .get_or_try_insert_async(asset_id, || {
                load_mesh_from_server(dist, &context.asset_store, &context.client)
            })
            .await?;

        Ok(Self {
            asset_id: Some(asset_id),
            label: dist.label.clone(),
            cpu: Some(cpu),
            gpu: PerBackend::default(),
        })
    }
}

async fn load_mesh_from_server<'a, 'b: 'a>(
    dist: &dist::Mesh,
    asset_store: &AssetStoreGuard,
    client: &AssetClient,
) -> Result<Arc<CpuMesh>, MeshError> {
    let mut file = asset_store
        .open(&dist.mesh, OpenOptions::new().create(true))
        .await?;

    let mut data = None;

    if !file.was_created() {
        let meta_data = file
            .meta_data()
            .get::<AssetStoreMetaData>("asset")?
            .unwrap_or_default();
        if meta_data.build_time.map_or(false, |t| t >= dist.build_time) {
            data = Some(file.read().await?);
        }
    }

    let data = if let Some(data) = data {
        data
    }
    else {
        let fetched_data = client.download_file(&dist.mesh).await?.bytes().await?;
        file.meta_data_mut().insert(
            "asset",
            &AssetStoreMetaData {
                asset_id: Some(dist.id),
                build_time: Some(dist.build_time),
            },
        )?;
        file.write(&fetched_data).await?;
        fetched_data
    };

    let mesh: CpuMesh = rmp_serde::from_slice(&data)?;
    Ok::<_, MeshError>(Arc::new(mesh))
}

fn load_mesh_to_gpu(
    mesh: &CpuMesh,
    label: Option<&str>,
    backend: &Backend,
) -> Result<GpuMesh, MeshError> {
    if mesh.primitive_topology != PrimitiveTopology::TriangleList {
        todo!(
            "trying to load mesh with incompatible primitive topology: {:?}",
            mesh.primitive_topology
        );
    }

    let vertex_buffer = backend
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: label
                .as_ref()
                .map(|l| format!("vertex buffer: {l}"))
                .as_deref(),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

    let index_buffer = backend
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: label
                .as_ref()
                .map(|l| format!("index buffer: {l}"))
                .as_deref(),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

    Ok(GpuMesh {
        vertex_buffer,
        index_buffer,
        num_indices: mesh.indices.len().try_into().unwrap(),
    })
}

impl From<CpuMesh> for Mesh {
    fn from(mesh: CpuMesh) -> Self {
        Mesh {
            asset_id: None,
            label: None,
            cpu: Some(Arc::new(mesh)),
            gpu: PerBackend::default(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("mesh load error")]
pub enum MeshError {
    AssetNotFound(#[from] AssetNotFound),
    Download(#[from] DownloadError),
    WebFs(#[from] web_fs::Error),
    Decode(#[from] rmp_serde::decode::Error),
    NoCpuMesh,
}

#[derive(Debug)]
pub struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl GpuMesh {
    pub fn id(&self) -> GpuMeshId {
        GpuMeshId {
            vertex: self.vertex_buffer.global_id(),
            index: self.index_buffer.global_id(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuMeshId {
    vertex: wgpu::Id<wgpu::Buffer>,
    index: wgpu::Id<wgpu::Buffer>,
}

pub trait Meshable {
    type Output: MeshBuilder;

    fn mesh(&self) -> Self::Output;
}

pub trait MeshBuilder {
    fn build(&self) -> CpuMesh;
}
