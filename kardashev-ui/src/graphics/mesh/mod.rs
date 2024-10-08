//pub mod teapot;
pub mod shape;

use std::sync::Arc;

use kardashev_client::DownloadError;
use kardashev_protocol::assets::{
    self as dist,
    AssetId,
};
pub use kardashev_protocol::assets::{
    MeshData,
    PrimitiveTopology,
    Vertex,
};
use wgpu::util::DeviceExt;

use super::loading::{
    GpuAsset,
    LoadContext,
};
use crate::{
    assets::{
        load::{
            LoadAssetContext,
            LoadFromAsset,
        },
        store::AssetStoreMetaData,
        AssetNotFound,
        MaybeHasAssetId,
    },
    utils::web_fs::{
        self,
        OpenOptions,
    },
};

#[derive(Debug)]
pub struct Mesh {
    pub asset_id: Option<AssetId>,
    pub label: Option<String>,
    pub mesh_data: Arc<MeshData>,
}

impl MaybeHasAssetId for Mesh {
    fn maybe_asset_id(&self) -> Option<AssetId> {
        self.asset_id
    }
}

impl LoadFromAsset for Mesh {
    type Dist = dist::Mesh;
    type Error = MeshLoadError;
    type Args = ();

    async fn load<'a, 'b: 'a>(
        asset_id: AssetId,
        _args: (),
        context: &'a mut LoadAssetContext<'b>,
    ) -> Result<Self, MeshLoadError> {
        let dist = context
            .dist_assets
            .get::<dist::Mesh>(asset_id)
            .ok_or_else(|| AssetNotFound { asset_id })?;

        let mesh_data = context
            .cache
            .get_or_try_insert_async(asset_id, || {
                async {
                    let mut file = context
                        .asset_store
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
                        let fetched_data = context
                            .client
                            .download_file(&dist.mesh)
                            .await?
                            .bytes()
                            .await?;
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

                    let mesh_data: MeshData = rmp_serde::from_slice(&data)?;
                    Ok::<_, MeshLoadError>(Arc::new(mesh_data))
                }
            })
            .await?;

        Ok(Self {
            asset_id: Some(asset_id),
            label: dist.label.clone(),
            mesh_data,
        })
    }
}

impl GpuAsset for Mesh {
    type Loaded = LoadedMesh;

    fn load(&self, context: &LoadContext) -> Result<Self::Loaded, super::Error> {
        if self.mesh_data.primitive_topology != PrimitiveTopology::TriangleList {
            todo!(
                "trying to load mesh with incompatible primitive topology: {:?}",
                self.mesh_data.primitive_topology
            );
        }

        let vertex_buffer =
            context
                .backend
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: self
                        .label
                        .as_ref()
                        .map(|l| format!("vertex buffer: {l}"))
                        .as_deref(),
                    contents: bytemuck::cast_slice(&self.mesh_data.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            context
                .backend
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: self
                        .label
                        .as_ref()
                        .map(|l| format!("index buffer: {l}"))
                        .as_deref(),
                    contents: bytemuck::cast_slice(&self.mesh_data.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        Ok(LoadedMesh {
            vertex_buffer,
            index_buffer,
            num_indices: self.mesh_data.indices.len().try_into().unwrap(),
        })
    }
}

impl From<MeshData> for Mesh {
    fn from(mesh_data: MeshData) -> Self {
        Mesh {
            asset_id: None,
            label: None,
            mesh_data: Arc::new(mesh_data),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("mesh load error")]
pub enum MeshLoadError {
    AssetNotFound(#[from] AssetNotFound),
    Download(#[from] DownloadError),
    WebFs(#[from] web_fs::Error),
    Decode(#[from] rmp_serde::decode::Error),
}

impl LoadedMesh {
    pub fn from_mesh_data(
        mesh_data: &MeshData,
        context: &LoadContext,
        label: Option<&str>,
    ) -> Self {
        let vertex_buffer =
            context
                .backend
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: label.map(|l| format!("vertex buffer: {l}")).as_deref(),
                    contents: bytemuck::cast_slice(&mesh_data.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            context
                .backend
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: label.map(|l| format!("index buffer: {l}")).as_deref(),
                    contents: bytemuck::cast_slice(&mesh_data.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        LoadedMesh {
            vertex_buffer,
            index_buffer,
            num_indices: mesh_data.indices.len().try_into().unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct LoadedMesh {
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
