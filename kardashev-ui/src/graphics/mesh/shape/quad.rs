use kardashev_protocol::assets::{
    MeshData,
    PrimitiveTopology,
    Vertex,
    WindingOrder,
};
use nalgebra::Vector2;

use crate::graphics::mesh::{
    MeshBuilder,
    Meshable,
};

#[derive(Clone, Copy, Debug)]
pub struct Quad {
    pub dimensions: Vector2<f32>,
}

impl Quad {
    pub fn new(dimensions: Vector2<f32>) -> Self {
        Self { dimensions }
    }
}

impl Default for Quad {
    fn default() -> Self {
        Self {
            dimensions: Vector2::repeat(1.0),
        }
    }
}

impl Meshable for Quad {
    type Output = RectangleMeshBuilder;

    fn mesh(&self) -> Self::Output {
        RectangleMeshBuilder { rectangle: *self }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RectangleMeshBuilder {
    pub rectangle: Quad,
}

impl MeshBuilder for RectangleMeshBuilder {
    fn build(&self) -> MeshData {
        MeshData {
            primitive_topology: PrimitiveTopology::TriangleList,
            winding_order: WindingOrder::CounterClockwise,
            has_binormals: false,
            indices: vec![0, 2, 1, 1, 2, 3],
            vertices: vec![
                Vertex {
                    position: [
                        -0.5 * self.rectangle.dimensions.x,
                        -0.5 * self.rectangle.dimensions.y,
                        0.,
                    ],
                    normal: [0., 0., 1.],
                    tex_coords: [0., 0.],
                    tangent: Default::default(),
                    bitangent: Default::default(),
                },
                Vertex {
                    position: [
                        0.5 * self.rectangle.dimensions.x,
                        -0.5 * self.rectangle.dimensions.y,
                        0.,
                    ],
                    normal: [0., 0., 1.],
                    tex_coords: [1., 0.],
                    tangent: Default::default(),
                    bitangent: Default::default(),
                },
                Vertex {
                    position: [
                        -0.5 * self.rectangle.dimensions.x,
                        0.5 * self.rectangle.dimensions.y,
                        0.,
                    ],
                    normal: [0., 0., 1.],
                    tex_coords: [0., 1.],
                    tangent: Default::default(),
                    bitangent: Default::default(),
                },
                Vertex {
                    position: [
                        0.5 * self.rectangle.dimensions.x,
                        0.5 * self.rectangle.dimensions.y,
                        0.,
                    ],
                    normal: [0., 0., 1.],
                    tex_coords: [1., 1.],
                    tangent: Default::default(),
                    bitangent: Default::default(),
                },
            ],
        }
        .with_binormals()
    }
}
