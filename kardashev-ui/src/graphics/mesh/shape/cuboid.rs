use kardashev_protocol::assets::{
    MeshData,
    PrimitiveTopology,
    Vertex,
    WindingOrder,
};
use nalgebra::Vector3;

use crate::graphics::mesh::{
    MeshBuilder,
    Meshable,
};

#[derive(Clone, Copy, Debug)]
pub struct Cuboid {
    pub dimensions: Vector3<f32>,
}

impl Default for Cuboid {
    fn default() -> Self {
        Self {
            dimensions: Vector3::repeat(1.0),
        }
    }
}

impl Meshable for Cuboid {
    type Output = CuboidMeshBuilder;

    fn mesh(&self) -> Self::Output {
        CuboidMeshBuilder { cuboid: *self }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CuboidMeshBuilder {
    pub cuboid: Cuboid,
}

impl MeshBuilder for CuboidMeshBuilder {
    fn build(&self) -> MeshData {
        // adapted from https://gist.github.com/prucha/866b9535d525adc984c4fe883e73a6c7

        #[rustfmt::skip]
        const BASE_VERTICES: [[f32; 3]; 8] = [
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, -1.0, -1.0],
            [-1.0, -1.0, -1.0],
            [-1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
        ];

        #[rustfmt::skip]
        const VERTICES: [[f32; 3]; 24] = [
	        BASE_VERTICES[0], BASE_VERTICES[1], BASE_VERTICES[2], BASE_VERTICES[3], // Bottom
	        BASE_VERTICES[7], BASE_VERTICES[4], BASE_VERTICES[0], BASE_VERTICES[3], // Left
	        BASE_VERTICES[4], BASE_VERTICES[5], BASE_VERTICES[1], BASE_VERTICES[0], // Front
	        BASE_VERTICES[6], BASE_VERTICES[7], BASE_VERTICES[3], BASE_VERTICES[2], // Back
	        BASE_VERTICES[5], BASE_VERTICES[6], BASE_VERTICES[2], BASE_VERTICES[1], // Right
	        BASE_VERTICES[7], BASE_VERTICES[6], BASE_VERTICES[5], BASE_VERTICES[4]  // Top
        ];

        fn pos(i: usize, dim: &Vector3<f32>) -> [f32; 3] {
            [
                VERTICES[i][0] * 0.5 * dim.x,
                VERTICES[i][1] * 0.5 * dim.y,
                VERTICES[i][2] * 0.5 * dim.z,
            ]
        }

        #[rustfmt::skip]
        const NORMALS: [[f32; 3]; 6] = [
            [0.0, -1.0, 0.0], // Bottom
            [-1.0, 0.0, 0.0], // Left
            [0.0, 0.0, 1.0], // Front
            [0.0, 0.0, -1.0], // Back
            [1.0, 0.0, 0.0], // Right
            [0.0, 1.0, 0.0], // Top
        ];

        #[rustfmt::skip]
        const BASE_UVS: [[f32; 2]; 4] = [
            [0.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
        ];

        #[rustfmt::skip]
        const UNFOLD_UVS: [[f32; 2]; 6] = [
            [1.0, 1.0], // bottom
            [0.0, 1.0], // left
            [1.0, 2.0], // front
            [1.0, 0.0], // back
            [2.0, 1.0], // right
            [1.0, 3.0], // top
        ];

        const fn add_uvs(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
            [(a[0] + b[0]) / 3.0, (a[1] + b[1]) / 4.0]
        }
        #[rustfmt::skip]
        const UVS: [[f32; 2]; 24] = [
            add_uvs(BASE_UVS[3], UNFOLD_UVS[0]), add_uvs(BASE_UVS[2], UNFOLD_UVS[0]), add_uvs(BASE_UVS[0], UNFOLD_UVS[0]), add_uvs(BASE_UVS[1], UNFOLD_UVS[0]), // Bottom
            add_uvs(BASE_UVS[3], UNFOLD_UVS[1]), add_uvs(BASE_UVS[2], UNFOLD_UVS[1]), add_uvs(BASE_UVS[0], UNFOLD_UVS[1]), add_uvs(BASE_UVS[1], UNFOLD_UVS[1]), // Left
            add_uvs(BASE_UVS[3], UNFOLD_UVS[2]), add_uvs(BASE_UVS[2], UNFOLD_UVS[2]), add_uvs(BASE_UVS[0], UNFOLD_UVS[2]), add_uvs(BASE_UVS[1], UNFOLD_UVS[2]), // Front
            add_uvs(BASE_UVS[3], UNFOLD_UVS[3]), add_uvs(BASE_UVS[2], UNFOLD_UVS[3]), add_uvs(BASE_UVS[0], UNFOLD_UVS[3]), add_uvs(BASE_UVS[1], UNFOLD_UVS[3]), // Back	        
            add_uvs(BASE_UVS[3], UNFOLD_UVS[4]), add_uvs(BASE_UVS[2], UNFOLD_UVS[4]), add_uvs(BASE_UVS[0], UNFOLD_UVS[4]), add_uvs(BASE_UVS[1], UNFOLD_UVS[4]), // Right 
            add_uvs(BASE_UVS[3], UNFOLD_UVS[5]), add_uvs(BASE_UVS[2], UNFOLD_UVS[5]), add_uvs(BASE_UVS[0], UNFOLD_UVS[5]), add_uvs(BASE_UVS[1], UNFOLD_UVS[5]), // Top
        ];

        #[rustfmt::skip]
        const INDICES: [u16; 36] = [
            3, 1, 0,        3, 2, 1,        // Bottom	
	        7, 5, 4,        7, 6, 5,        // Left
	        11, 9, 8,       11, 10, 9,      // Front
	        15, 13, 12,     15, 14, 13,     // Back
	        19, 17, 16,     19, 18, 17,	    // Right
	        23, 21, 20,     23, 22, 21,	    // Top
        ];

        MeshData {
            primitive_topology: PrimitiveTopology::TriangleList,
            winding_order: WindingOrder::CounterClockwise,
            has_binormals: false,
            indices: INDICES.into_iter().collect(),
            vertices: (0..24)
                .map(|i| {
                    Vertex {
                        position: pos(i, &self.cuboid.dimensions),
                        normal: NORMALS[i / 4],
                        tex_coords: UVS[i],
                        tangent: Default::default(),
                        bitangent: Default::default(),
                    }
                })
                .collect(),
        }
        .with_binormals()
    }
}
