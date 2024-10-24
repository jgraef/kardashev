use kardashev_protocol::assets::{
    MeshData,
    PrimitiveTopology,
    Vertex,
    WindingOrder,
};
use nalgebra::{
    Vector2,
    Vector3,
};

use super::{
    MeshBuilder,
    Meshable,
};

#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub radius: f32,
}

impl Sphere {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self::new(1.)
    }
}

impl Meshable for Sphere {
    type Output = SphereMeshBuilder;

    fn mesh(&self) -> Self::Output {
        SphereMeshBuilder {
            sphere: self.clone(),
            mesh_type: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SphereMeshType {
    Ico { subdivisions: usize },
    Uv { sectors: usize, stacks: usize },
}

impl Default for SphereMeshType {
    fn default() -> Self {
        Self::Uv {
            sectors: 32,
            stacks: 18,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SphereMeshBuilder {
    pub sphere: Sphere,
    pub mesh_type: SphereMeshType,
}

impl MeshBuilder for SphereMeshBuilder {
    fn build(&self) -> MeshData {
        match self.mesh_type {
            SphereMeshType::Ico { subdivisions: _ } => todo!(),
            SphereMeshType::Uv { sectors, stacks } => {
                mesh_sphere_uv(self.sphere.radius, sectors, stacks)
            }
        }
    }
}

fn mesh_sphere_uv(radius: f32, sectors: usize, stacks: usize) -> MeshData {
    // taken from bevy: https://github.com/bevyengine/bevy/blob/39d96ef0fd1afd0b63edda979903a32815383038/crates/bevy_render/src/mesh/primitives/dim3/sphere.rs#L169

    use std::f32::consts::PI;

    let sectors_f32 = sectors as f32;
    let stacks_f32 = stacks as f32;
    let length_inv = 1. / radius;
    let sector_step = 2. * PI / sectors_f32;
    let stack_step = PI / stacks_f32;

    let n_vertices = stacks * sectors;
    let mut vertices: Vec<Vertex> = Vec::with_capacity(n_vertices);
    let mut indices: Vec<u16> = Vec::with_capacity(n_vertices * 2 * 3);

    for i in 0..stacks + 1 {
        let stack_angle = PI / 2. - (i as f32) * stack_step;
        let xy = radius * stack_angle.cos();
        let z = radius * stack_angle.sin();

        for j in 0..sectors + 1 {
            let sector_angle = (j as f32) * sector_step;
            let x = xy * sector_angle.cos();
            let y = xy * sector_angle.sin();

            vertices.push(Vertex {
                position: [x, y, z],
                normal: [x * length_inv, y * length_inv, z * length_inv],
                tex_coords: [(j as f32) / sectors_f32, (i as f32) / stacks_f32],
            });
        }
    }

    // indices
    //  k1--k1+1
    //  |  / |
    //  | /  |
    //  k2--k2+1
    for i in 0..stacks {
        let mut k1 = i * (sectors + 1);
        let mut k2 = k1 + sectors + 1;
        for _j in 0..sectors {
            if i != 0 {
                indices.push(k1.try_into().unwrap());
                indices.push(k2.try_into().unwrap());
                indices.push((k1 + 1).try_into().unwrap());
            }
            if i != stacks - 1 {
                indices.push((k1 + 1).try_into().unwrap());
                indices.push(k2.try_into().unwrap());
                indices.push((k2 + 1).try_into().unwrap());
            }
            k1 += 1;
            k2 += 1;
        }
    }

    MeshData {
        primitive_topology: PrimitiveTopology::TriangleList,
        winding_order: WindingOrder::CounterClockwise,
        indices,
        vertices,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    pub dimensions: Vector2<f32>,
}

impl Rectangle {
    pub fn new(dimensions: Vector2<f32>) -> Self {
        Self { dimensions }
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            dimensions: Vector2::repeat(1.0),
        }
    }
}

impl Meshable for Rectangle {
    type Output = RectangleMeshBuilder;

    fn mesh(&self) -> Self::Output {
        RectangleMeshBuilder { rectangle: *self }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RectangleMeshBuilder {
    pub rectangle: Rectangle,
}

impl MeshBuilder for RectangleMeshBuilder {
    fn build(&self) -> MeshData {
        MeshData {
            primitive_topology: PrimitiveTopology::TriangleList,
            winding_order: WindingOrder::CounterClockwise,
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
                },
                Vertex {
                    position: [
                        0.5 * self.rectangle.dimensions.x,
                        -0.5 * self.rectangle.dimensions.y,
                        0.,
                    ],
                    normal: [0., 0., 1.],
                    tex_coords: [1., 0.],
                },
                Vertex {
                    position: [
                        -0.5 * self.rectangle.dimensions.x,
                        0.5 * self.rectangle.dimensions.y,
                        0.,
                    ],
                    normal: [0., 0., 1.],
                    tex_coords: [0., 1.],
                },
                Vertex {
                    position: [
                        0.5 * self.rectangle.dimensions.x,
                        0.5 * self.rectangle.dimensions.y,
                        0.,
                    ],
                    normal: [0., 0., 1.],
                    tex_coords: [1., 1.],
                },
            ],
        }
    }
}

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
            indices: INDICES.into_iter().collect(),
            vertices: (0..24)
                .map(|i| {
                    Vertex {
                        position: pos(i, &self.cuboid.dimensions),
                        normal: NORMALS[i / 4],
                        tex_coords: UVS[i],
                    }
                })
                .collect(),
        }
    }
}
