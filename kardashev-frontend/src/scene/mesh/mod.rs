pub mod teapot;

use nalgebra::{
    Vector2,
    Vector3,
};

#[derive(Clone, Debug)]
pub struct Mesh {
    primitive_topology: PrimitiveTopology,
    indices: Vec<usize>,
    positions: Vec<Vector3<f32>>,
    normals: Vec<Vector3<f32>>,
    uv: Vec<Vector2<f32>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
}
