use kardashev_protocol::assets::{
    PrimitiveTopology,
    Vertex,
};

use super::{
    MeshBuilder,
    MeshData,
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
        indices,
        vertices,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            width: 1.,
            height: 1.,
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
            indices: vec![0, 2, 1, 1, 2, 3],
            vertices: vec![
                Vertex {
                    position: [
                        -0.5 * self.rectangle.width,
                        -0.5 * self.rectangle.height,
                        0.,
                    ],
                    normal: [0., 0., 1.],
                    tex_coords: [0., 0.],
                },
                Vertex {
                    position: [0.5 * self.rectangle.width, -0.5 * self.rectangle.height, 0.],
                    normal: [0., 0., 1.],
                    tex_coords: [1., 0.],
                },
                Vertex {
                    position: [-0.5 * self.rectangle.width, 0.5 * self.rectangle.height, 0.],
                    normal: [0., 0., 1.],
                    tex_coords: [0., 1.],
                },
                Vertex {
                    position: [0.5 * self.rectangle.width, 0.5 * self.rectangle.height, 0.],
                    normal: [0., 0., 1.],
                    tex_coords: [1., 1.],
                },
            ],
        }
    }
}
