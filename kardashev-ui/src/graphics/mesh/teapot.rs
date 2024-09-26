use std::io::Cursor;

use lazy_static::lazy_static;

use super::{
    Mesh,
    MeshData,
    PrimitiveTopology,
};

static TEAPOT_OBJ: &[u8] = include_bytes!("teapot.obj");
static TEAPOT_MTL: &[u8] = include_bytes!("teapot.mtl");

lazy_static! {
    pub static ref TEAPOT_MESH: Mesh = load_mesh(TEAPOT_OBJ, TEAPOT_MTL);
}

fn load_mesh(obj_data: &[u8], mtl_data: &[u8]) -> MeshData {
    let (model, mtl_load_result) = tobj::load_obj_buf(&mut Cursor::new(TEAPOT_OBJ), &Default::default(), |_path| tobj::load_mtl_buf(&mut Cursor::new(mtl_data))).unwrap();
    let materials = mtl_load_result.unwrap();

    MeshData {
        primitive_topology: PrimitiveTopology::TriangleList,
        indices: model.mesh.indices,
        positions: obj.vertices.iter().map(|v| v.position.into()).collect(),
        normals: obj.vertices.iter().map(|v| v.normal.into()).collect(),
        tex_coords: vec![],
    }
}
