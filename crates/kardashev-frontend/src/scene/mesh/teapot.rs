use lazy_static::lazy_static;

use super::{
    Mesh,
    PrimitiveTopology,
};

static TEAPOT_OBJ: &str = include_str!("teapot.obj");
//static TEAPOT_MTL: &str = include_str!("teapot.mtl");

lazy_static! {
    pub static ref TEAPOT_MESH: Mesh = {
        let obj = obj::load_obj::<obj::Vertex, _, usize>(TEAPOT_OBJ.as_bytes())
            .expect("invalid teapot.obj");

        Mesh {
            primitive_topology: PrimitiveTopology::TriangleList,
            indices: obj.indices,
            positions: obj.vertices.iter().map(|v| v.position.into()).collect(),
            normals: obj.vertices.iter().map(|v| v.normal.into()).collect(),
            uv: vec![],
        }
    };
}
