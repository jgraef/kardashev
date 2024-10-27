use kardashev_protocol::assets::{
    MeshData,
    PrimitiveTopology,
    Vertex,
    WindingOrder,
};
use nalgebra::Vector3;

pub mod cuboid;
pub mod quad;
pub mod sphere;

#[derive(Clone, Debug)]
pub struct MeshDataBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    winding_order: WindingOrder,
}

impl Default for MeshDataBuilder {
    fn default() -> Self {
        Self::with_capacity(0, 0)
    }
}

impl MeshDataBuilder {
    pub fn with_capacity(vertices: usize, indices: usize) -> Self {
        Self {
            indices: Vec::with_capacity(vertices),
            vertices: Vec::with_capacity(indices),
            winding_order: WindingOrder::CounterClockwise,
        }
    }

    pub fn add_vertex(&mut self, vertex: Vertex) -> u16 {
        let index = self.vertices.len();
        self.vertices.push(vertex);
        index
            .try_into()
            .expect("vertex index out of bounds for u16")
    }

    pub fn add_face<F: Face>(&mut self, face: F) {
        face.add_indices(self);
    }

    pub fn build(self) -> MeshData {
        MeshData {
            primitive_topology: PrimitiveTopology::TriangleList,
            winding_order: self.winding_order,
            has_binormals: false,
            indices: self.indices,
            vertices: self.vertices,
        }
        .with_binormals()
    }

    fn get_vertex(&self, vertex_index: u16) -> &Vertex {
        &self.vertices[usize::from(vertex_index)]
    }
}

pub trait Face {
    fn add_indices(&self, builder: &mut MeshDataBuilder);
    fn normal(&self, builder: &mut MeshDataBuilder) -> Vector3<f32>;
}

impl Face for [u16; 3] {
    fn add_indices(&self, builder: &mut MeshDataBuilder) {
        builder.indices.push(self[0]);
        builder.indices.push(self[1]);
        builder.indices.push(self[2]);
    }

    fn normal(&self, builder: &mut MeshDataBuilder) -> Vector3<f32> {
        let v0 = Vector3::from(builder.get_vertex(self[0]).position);
        let v1 = Vector3::from(builder.get_vertex(self[1]).position);
        let v2 = Vector3::from(builder.get_vertex(self[2]).position);
        let e01 = v1 - v0;
        let e02 = v2 - v0;
        e01.cross(&e02)
    }
}

impl Face for [u16; 4] {
    fn add_indices(&self, builder: &mut MeshDataBuilder) {
        builder.indices.push(self[0]);
        builder.indices.push(self[1]);
        builder.indices.push(self[2]);
        builder.indices.push(self[0]);
        builder.indices.push(self[2]);
        builder.indices.push(self[3]);
    }

    fn normal(&self, builder: &mut MeshDataBuilder) -> Vector3<f32> {
        let v0 = Vector3::from(builder.get_vertex(self[0]).position);
        let v1 = Vector3::from(builder.get_vertex(self[1]).position);
        let v3 = Vector3::from(builder.get_vertex(self[3]).position);
        let e01 = v1 - v0;
        let e03 = v3 - v0;
        e01.cross(&e03)
    }
}
