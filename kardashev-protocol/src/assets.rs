use std::{
    any::{
        type_name,
        Any,
    },
    borrow::Cow,
    collections::{
        HashMap,
        HashSet,
    },
    fmt::{
        Debug,
        Display,
    },
    hash::Hash,
    marker::PhantomData,
    ops::Deref,
};

use bytemuck::{
    Pod,
    Zeroable,
};
use chrono::{
    DateTime,
    Utc,
};
use nalgebra::{
    Vector2,
    Vector3,
};
use palette::Srgb;
use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};
use uuid::{
    uuid,
    Uuid,
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct AssetId(Uuid);

impl AssetId {
    pub fn generate() -> Self {
        Self::from_uuid(Uuid::new_v4())
    }

    #[doc(hidden)]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[macro_export]
macro_rules! asset_id {
    ($lit:literal) => {
        ::kardashev_protocol::assets::AssetId::from_uuid(::kardashev_protocol::uuid::uuid!($lit))
    };
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub build_time: DateTime<Utc>,
    pub assets: AssetsBlob,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Texture {
    pub id: AssetId,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    pub build_time: DateTime<Utc>,

    pub image: String,

    pub size: TextureSize,

    #[serde(default)]
    pub format: TextureFormat,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub crop: Option<TextureCrop>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub u_edge_mode: Option<TextureEdgeMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_edge_mode: Option<TextureEdgeMode>,
}

impl HasAssetId for Texture {
    fn asset_id(&self) -> AssetId {
        self.id
    }
}

impl Asset for Texture {
    const TYPE_NAME: &'static str = "texture";
    const TYPE_ID: Uuid = uuid!("f4c83063-accc-4565-82a9-04df9582ec69");

    fn files<'a>(&'a self) -> impl Iterator<Item = &'a str> {
        std::iter::once(&*self.image)
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextureFormat {
    #[default]
    Rgba8UnormSrgb,
    Rgba8Unorm,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextureSize {
    pub w: u32,
    pub h: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextureCrop {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TextureEdgeMode {
    ClampToEdge,
    Repeat,
    MirrorRepeat,
    ClampToBorder,
}

// todo: how do we handle different materials here?
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Material {
    pub id: AssetId,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    pub build_time: DateTime<Utc>,

    // both
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal_texture: Option<AssetId>,

    // blinn-phong
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambient_texture: Option<AssetId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambient_color: Option<Srgb<f32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub diffuse_texture: Option<AssetId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub diffuse_color: Option<Srgb<f32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub specular_texture: Option<AssetId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub specular_color: Option<Srgb<f32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shininess_texture: Option<AssetId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shininess: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolve_texture: Option<AssetId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolve: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub emissive_texture: Option<AssetId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub emissive_color: Option<Srgb<f32>>,

    // pbr
    // todo: colors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub albedo_texture: Option<AssetId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metalness_texture: Option<AssetId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness_texture: Option<AssetId>,
}

impl HasAssetId for Material {
    fn asset_id(&self) -> AssetId {
        self.id
    }
}

impl Asset for Material {
    const TYPE_NAME: &'static str = "material";
    const TYPE_ID: Uuid = uuid!("ec98ef77-e2ce-4cc8-baf2-28cf53b88171");

    fn files<'a>(&'a self) -> impl Iterator<Item = &'a str> {
        std::iter::empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub id: AssetId,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    pub build_time: DateTime<Utc>,

    pub mesh: String,
}

impl HasAssetId for Mesh {
    fn asset_id(&self) -> AssetId {
        self.id
    }
}

impl Asset for Mesh {
    const TYPE_NAME: &'static str = "mesh";
    const TYPE_ID: Uuid = uuid!("15668e5b-73aa-4895-8c70-3cf0346251eb");

    fn files<'a>(&'a self) -> impl Iterator<Item = &'a str> {
        std::iter::once(&*self.mesh)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshData {
    pub primitive_topology: PrimitiveTopology,
    pub winding_order: WindingOrder,
    pub has_binormals: bool,
    pub indices: Vec<u16>,
    pub vertices: Vec<Vertex>,
}

impl MeshData {
    pub fn with_binormals(mut self) -> Self {
        // taken straight from the [wgpu tutorial][1]
        // [1]: https://sotrh.github.io/learn-wgpu/intermediate/tutorial11-normals/#the-tangent-and-the-bitangent

        if !self.has_binormals {
            let mut triangles_included = vec![0; self.vertices.len()];

            // Calculate tangents and bitangets. We're going to
            // use the triangles, so we need to loop through the
            // indices in chunks of 3
            for c in self.indices.chunks(3) {
                let v0 = self.vertices[c[0] as usize];
                let v1 = self.vertices[c[1] as usize];
                let v2 = self.vertices[c[2] as usize];

                let pos0: Vector3<f32> = v0.position.into();
                let pos1: Vector3<f32> = v1.position.into();
                let pos2: Vector3<f32> = v2.position.into();

                let uv0: Vector2<f32> = v0.tex_coords.into();
                let uv1: Vector2<f32> = v1.tex_coords.into();
                let uv2: Vector2<f32> = v2.tex_coords.into();

                // Calculate the edges of the triangle
                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                // This will give us a direction to calculate the
                // tangent and bitangent
                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                // Solving the following system of equations will
                // give us the tangent and bitangent.
                //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                // Luckily, the place I found this equation provided
                // the solution!
                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                // We flip the bitangent to enable right-handed normal
                // maps with wgpu texture coordinate system
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                // We'll use the same tangent/bitangent for each vertex in the triangle
                self.vertices[c[0] as usize].tangent =
                    (tangent + Vector3::from(self.vertices[c[0] as usize].tangent)).into();
                self.vertices[c[1] as usize].tangent =
                    (tangent + Vector3::from(self.vertices[c[1] as usize].tangent)).into();
                self.vertices[c[2] as usize].tangent =
                    (tangent + Vector3::from(self.vertices[c[2] as usize].tangent)).into();
                self.vertices[c[0] as usize].bitangent =
                    (bitangent + Vector3::from(self.vertices[c[0] as usize].bitangent)).into();
                self.vertices[c[1] as usize].bitangent =
                    (bitangent + Vector3::from(self.vertices[c[1] as usize].bitangent)).into();
                self.vertices[c[2] as usize].bitangent =
                    (bitangent + Vector3::from(self.vertices[c[2] as usize].bitangent)).into();

                // Used to average the tangents/bitangents
                triangles_included[c[0] as usize] += 1;
                triangles_included[c[1] as usize] += 1;
                triangles_included[c[2] as usize] += 1;
            }

            // Average the tangents/bitangents
            for (i, n) in triangles_included.into_iter().enumerate() {
                let denom = 1.0 / n as f32;
                let v = &mut self.vertices[i];
                v.tangent = (Vector3::from(v.tangent) * denom).into();
                v.bitangent = (Vector3::from(v.bitangent) * denom).into();
            }

            self.has_binormals = true;
        }

        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindingOrder {
    Clockwise,
    CounterClockwise,
}

#[derive(Clone, Copy, Debug, Zeroable, Pod, Serialize, Deserialize)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Changed { asset_ids: Vec<AssetId> },
    Lagged,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shader {
    pub id: AssetId,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    pub build_time: DateTime<Utc>,

    pub naga_ir: String,
}

impl HasAssetId for Shader {
    fn asset_id(&self) -> AssetId {
        self.id
    }
}

impl Asset for Shader {
    const TYPE_NAME: &'static str = "shader";
    const TYPE_ID: Uuid = uuid!("ae943412-b95a-4097-8441-6e5a58905655");

    fn files<'a>(&'a self) -> impl Iterator<Item = &'a str> {
        std::iter::once(&*self.naga_ir)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledShader {
    pub label: Option<String>,
    pub module: naga::Module,
    pub module_info: naga::valid::ModuleInfo,
}

pub trait HasAssetId {
    fn asset_id(&self) -> AssetId;
}

pub trait Asset: HasAssetId + Serialize + DeserializeOwned + Send + Sync + 'static {
    const TYPE_NAME: &'static str;
    const TYPE_ID: Uuid;

    fn files<'a>(&'a self) -> impl Iterator<Item = &'a str>;
}

#[derive(Default)]
pub struct Assets {
    assets: HashMap<AssetId, (Box<dyn Any + Send + Sync + 'static>, DynAssetType)>,
    unrecognized: Vec<AssetBlob>,
}

impl Assets {
    pub fn get<A: Asset>(&self, asset_id: AssetId) -> Option<&A> {
        let asset = &self.assets.get(&asset_id)?.0;
        asset.downcast_ref()
    }

    pub fn insert<A: Asset>(&mut self, asset: A) {
        self.assets.insert(
            asset.asset_id(),
            (Box::new(asset), DynAssetType::new::<A>()),
        );
    }

    pub fn blob(&self) -> AssetsBlob {
        let mut blob = AssetsBlob::default();
        for (id, (asset, asset_type)) in &self.assets {
            let data = asset_type.serialize(&**asset).unwrap_or_else(|error| {
                panic!(
                    "Failed to serialize asset ({:?}): {error}",
                    asset_type.asset_type()
                )
            });
            blob.list.push(AssetBlob {
                id: *id,
                r#type: asset_type.asset_type(),
                data,
            });
        }
        blob
    }

    pub fn unrecognized_types(&self) -> HashSet<&AssetType> {
        self.unrecognized.iter().map(|blob| &blob.r#type).collect()
    }

    pub fn all_files(&self) -> HashSet<&str> {
        let mut files = HashSet::new();
        for (asset, asset_type) in self.assets.values() {
            asset_type.collect_files(&**asset, &mut files);
        }
        files
    }

    pub fn all_asset_ids(&self) -> impl Iterator<Item = AssetId> + '_ {
        self.assets.keys().copied()
    }

    pub fn remove(&mut self, asset_id: AssetId) {
        self.assets.remove(&asset_id);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetType {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Cow<'static, str>>,
}

impl Eq for AssetType {}

impl PartialEq for AssetType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for AssetType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, thiserror::Error)]
#[error("asset parse error")]
pub struct AssetParseError {
    #[source]
    pub source: serde_json::Error,
    pub asset_type: AssetType,
    pub asset_id: AssetId,
}

impl Debug for Assets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Assets").finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Default)]
pub struct AssetTypes {
    types: HashMap<Uuid, DynAssetType>,
}

impl AssetTypes {
    pub fn register<A: Asset>(&mut self) -> &mut Self {
        tracing::debug!(id = %A::TYPE_ID, name = A::TYPE_NAME, r#type = type_name::<A>(), "register asset type");
        self.types.insert(A::TYPE_ID, DynAssetType::new::<A>());
        self
    }

    pub fn with_builtin(&mut self) -> &mut Self {
        self.register::<Texture>();
        self.register::<Material>();
        self.register::<Mesh>();
        self.register::<Shader>();
        self
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AssetsBlob {
    list: Vec<AssetBlob>,
}

impl AssetsBlob {
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn parse(self, asset_types: &AssetTypes) -> Result<Assets, AssetParseError> {
        let mut assets = Assets::default();

        for asset in self.list {
            if let Some(asset_type) = asset_types.types.get(&asset.r#type.id) {
                assets.assets.insert(
                    asset.id,
                    (
                        asset_type.deserialize(&asset.data).map_err(|source| {
                            AssetParseError {
                                source,
                                asset_type: asset_type.asset_type(),
                                asset_id: asset.id,
                            }
                        })?,
                        *asset_type,
                    ),
                );
            }
            else {
                assets.unrecognized.push(asset);
            }
        }

        Ok(assets)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AssetBlob {
    id: AssetId,
    r#type: AssetType,
    data: serde_json::Value,
}

#[derive(Clone, Copy)]
struct DynAssetType {
    inner: &'static dyn DynAssetTypeTrait,
}

impl Debug for DynAssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynAssetType")
            .field("asset_type", &self.inner.asset_type())
            .finish()
    }
}

impl DynAssetType {
    pub fn new<A: Asset>() -> Self {
        Self {
            inner: &DynAssetTypeImpl {
                _ty: PhantomData::<A>,
            },
        }
    }
}

impl Deref for DynAssetType {
    type Target = dyn DynAssetTypeTrait;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

trait DynAssetTypeTrait: Send + Sync + 'static {
    fn asset_type(&self) -> AssetType;
    fn serialize(
        &self,
        asset: &(dyn Any + Send + Sync + 'static),
    ) -> Result<serde_json::Value, serde_json::Error>;
    fn deserialize(
        &self,
        data: &serde_json::Value,
    ) -> Result<Box<dyn Any + Send + Sync + 'static>, serde_json::Error>;
    fn collect_files<'a>(&self, asset: &'a dyn Any, files: &mut HashSet<&'a str>);
}

struct DynAssetTypeImpl<A> {
    _ty: PhantomData<A>,
}

impl<A: Asset> DynAssetTypeTrait for DynAssetTypeImpl<A> {
    fn asset_type(&self) -> AssetType {
        AssetType {
            id: A::TYPE_ID,
            name: Some(A::TYPE_NAME.into()),
        }
    }

    fn serialize(
        &self,
        asset: &(dyn Any + Send + Sync + 'static),
    ) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(asset.downcast_ref::<A>().unwrap())
    }

    fn deserialize(
        &self,
        data: &serde_json::Value,
    ) -> Result<Box<dyn Any + Send + Sync + 'static>, serde_json::Error> {
        Ok(Box::new(A::deserialize(data)?))
    }

    fn collect_files<'a>(&self, asset: &'a dyn Any, files: &mut HashSet<&'a str>) {
        files.extend(A::files(asset.downcast_ref::<A>().unwrap()));
    }
}
