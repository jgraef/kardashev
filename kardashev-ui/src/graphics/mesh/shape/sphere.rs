use std::f32::consts::{
    PI,
    TAU,
};

use kardashev_protocol::assets::{
    MeshData,
    PrimitiveTopology,
    Vertex,
    WindingOrder,
};
use nalgebra::Vector3;

use crate::graphics::mesh::{
    shape::MeshDataBuilder,
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
    Cube { subdivisions: usize },
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

impl SphereMeshBuilder {
    pub fn with_mesh_type(&mut self, mesh_type: SphereMeshType) -> &mut Self {
        self.mesh_type = mesh_type;
        self
    }
}

impl MeshBuilder for SphereMeshBuilder {
    fn build(&self) -> MeshData {
        match self.mesh_type {
            SphereMeshType::Cube { subdivisions } => {
                mesh_hexasphere(
                    self.sphere.radius,
                    subdivisions,
                    hexasphere::shapes::CubeBase::default(),
                )
            }
            SphereMeshType::Ico { subdivisions } => {
                mesh_hexasphere(
                    self.sphere.radius,
                    subdivisions,
                    hexasphere::shapes::IcoSphereBase::default(),
                )
            }
            SphereMeshType::Uv { sectors, stacks } => {
                mesh_sphere_uv(self.sphere.radius, sectors, stacks)
            }
        }
    }
}

fn mesh_hexasphere<B: hexasphere::BaseShape>(
    radius: f32,
    subdivisions: usize,
    base: B,
) -> MeshData {
    let generated = hexasphere::Subdivided::<_, B>::new_custom_shape(
        subdivisions,
        |point| {
            use std::f32::consts::{
                PI,
                TAU,
            };
            let u = 0.5 - point.y.atan2(-point.x) / TAU;
            let v = point.z.acos() / PI;
            [u, v]
        },
        base,
    );

    let raw_points = generated.raw_points();
    let raw_data = generated.raw_data();

    let vertices = raw_points
        .iter()
        .zip(raw_data)
        .map(|(point, uv)| {
            Vertex {
                position: (point * radius).into(),
                tex_coords: (*uv).into(),
                normal: (*point).into(),
                tangent: Default::default(),
                bitangent: Default::default(),
            }
        })
        .collect::<Vec<_>>();

    let indices = generated
        .get_all_indices()
        .into_iter()
        .map(|index| index.try_into().unwrap())
        .collect();

    MeshData {
        primitive_topology: PrimitiveTopology::TriangleList,
        winding_order: WindingOrder::CounterClockwise,
        has_binormals: false,
        indices,
        vertices,
    }
    .with_binormals()
}

fn mesh_sphere_uv(radius: f32, num_slices: usize, num_stacks: usize) -> MeshData {
    // https://danielsieger.com/blog/2021/03/27/generating-spheres.html

    assert!(num_stacks >= 2);
    assert!(num_slices >= 3);

    let num_vertices = num_slices * num_stacks;
    let num_indices = num_vertices * 6;

    let mut builder = MeshDataBuilder::with_capacity(num_vertices, num_indices);

    let stack_step_tex = 1.0 / (num_stacks as f32);
    let slice_step_tex = 1.0 / (num_slices as f32);
    let stack_step_angle = PI * stack_step_tex;
    let slice_step_angle = -TAU * slice_step_tex;

    let num_stacks = u16::try_from(num_stacks).unwrap();
    let num_slices = u16::try_from(num_slices).unwrap();

    // add top vertex
    let top_vertex = builder.add_vertex(Vertex {
        position: [0.0, radius, 0.0],
        tex_coords: [0.5, 0.0],
        normal: [0.0, 1.0, 0.0],
        tangent: Default::default(),
        bitangent: Default::default(),
    });

    // add middle vertices
    for i in 0..(num_stacks - 1) {
        let i = i as f32;
        let phi = i * stack_step_angle;

        for j in 0..(num_slices + 1) {
            let j = j as f32;
            let theta = j * slice_step_angle;

            let normal = Vector3::new(phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin());

            builder.add_vertex(Vertex {
                position: (normal * radius).into(),
                tex_coords: [j * slice_step_tex, i * stack_step_tex],
                normal: normal.into(),
                tangent: Default::default(),
                bitangent: Default::default(),
            });
        }
    }

    // add bottom vertex
    let bottom_vertex = builder.add_vertex(Vertex {
        position: [0.0, -radius, 0.0],
        tex_coords: [0.5, 0.0],
        normal: [0.0, -1.0, 0.0],
        tangent: Default::default(),
        bitangent: Default::default(),
    });

    // add top and bottom triangles
    for j in 0..num_slices {
        // first vertex of bottom stack
        let l = (num_stacks - 2) * (num_slices + 1) + 1;

        // top vertices
        let top1 = j + 1;
        let top2 = j + 2;

        builder.add_face([top_vertex, top1, top2]);

        // bottom vertices
        let bottom1 = j + l;
        let bottom2 = j + l + 1;

        builder.add_face([bottom1, bottom2, bottom_vertex]);
    }

    // add quads for each stack / slice
    for i in 0..(num_stacks - 2) {
        // first vertex of this stack
        let l1 = i * (num_slices + 1) + 1;
        // first vertex of the next stack
        let l2 = (i + 1) * (num_slices + 1) + 1;

        for j in 0..num_slices {
            let top_left = l1 + j;
            let top_right = l1 + j + 1;
            let bottom_left = l2 + j;
            let bottom_right = l2 + j + 1;

            builder.add_face([top_left, bottom_left, bottom_right, top_right]);
        }
    }

    builder.build()
}
