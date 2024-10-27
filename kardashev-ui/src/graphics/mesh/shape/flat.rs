use std::f32::consts::TAU;

use kardashev_protocol::assets::{
    MeshData,
    Vertex,
};
use nalgebra::Vector2;

use crate::graphics::mesh::{
    shape::builder::{
        ShapeBuilder,
        Triangulate,
    },
    MeshBuilder,
    Meshable,
};

// normal for flat shapes
const NORMAL: [f32; 3] = [0.0, 0.0, 1.0];

#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    pub half_size: Vector2<f32>,
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            half_size: Vector2::repeat(0.5),
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
        let mut builder = ShapeBuilder::with_capacity(4, 6);

        // vertices (pos, uv)
        const BASE: [([f32; 2], [f32; 2]); 4] = [
            ([-1.0, 1.0], [0.0, 0.0]),
            ([-1.0, -1.0], [1.0, 0.0]),
            ([1.0, -1.0], [0.0, 1.0]),
            ([1.0, 1.0], [1.0, 1.0]),
        ];

        let vertices = BASE.map(|(pos, tex_coords)| {
            builder.add_vertex(Vertex {
                position: [
                    pos[0] * self.rectangle.half_size.x,
                    pos[1] * self.rectangle.half_size.y,
                    0.,
                ],
                normal: NORMAL,
                tex_coords,
                tangent: Default::default(),
                bitangent: Default::default(),
            })
        });
        builder.add_face(vertices);

        builder.build()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Circle {
    pub radius: f32,
}

impl Default for Circle {
    fn default() -> Self {
        Circle { radius: 0.5 }
    }
}

impl Meshable for Circle {
    type Output = CircleMeshBuilder;

    fn mesh(&self) -> Self::Output {
        CircleMeshBuilder {
            circle: *self,
            mesh_type: CircleMeshType::Segments { segments: 36 },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CircleMeshBuilder {
    pub circle: Circle,
    pub mesh_type: CircleMeshType,
}

#[derive(Clone, Copy, Debug)]
pub enum CircleMeshType {
    Segments { segments: usize },
    Polygon { vertices: usize },
}

impl MeshBuilder for CircleMeshBuilder {
    fn build(&self) -> MeshData {
        match self.mesh_type {
            CircleMeshType::Segments { segments } => {
                mesh_circle_segments(self.circle.radius, segments)
            }
            CircleMeshType::Polygon { vertices } => {
                mesh_circle_polygon(self.circle.radius, vertices)
            }
        }
    }
}

fn circle_vertices(radius: f32, vertices: usize) -> impl Iterator<Item = Vertex> {
    let theta_step = TAU / vertices as f32;
    (0..vertices).map(move |i| {
        let theta = theta_step * i as f32;

        let pos = Vector2::new(theta.cos(), -theta.sin());
        let tex_coords = (Vector2::repeat(0.5) + 0.5 * pos).into();

        Vertex {
            position: [pos.x * radius, pos.y * radius, 0.0],
            tex_coords,
            normal: NORMAL,
            tangent: Default::default(),
            bitangent: Default::default(),
        }
    })
}

fn mesh_circle_segments(radius: f32, segments: usize) -> MeshData {
    assert!(segments >= 3);

    let mut builder = ShapeBuilder::with_capacity(segments + 1, segments * 2);

    let center = builder.add_vertex(Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.5, 0.5],
        normal: NORMAL,
        tangent: Default::default(),
        bitangent: Default::default(),
    });

    let mut outer_vertices = circle_vertices(radius, segments);
    let mut previous = builder.add_vertex(outer_vertices.next().unwrap());

    for vertex in outer_vertices {
        let vertex = builder.add_vertex(vertex);
        builder.add_face([center, previous, vertex]);
        previous = vertex;
    }

    builder.build()
}

fn mesh_circle_polygon(radius: f32, vertices: usize) -> MeshData {
    assert!(vertices >= 3);

    let triangulate = Triangulate::new(vertices);
    let mut builder = ShapeBuilder::with_capacity(vertices, triangulate.len() * 3);

    for vertex in circle_vertices(radius, vertices) {
        builder.add_vertex(vertex);
    }

    for tri in Triangulate::new(vertices) {
        builder.add_tri(tri.map(|i| i.try_into().unwrap()));
    }

    builder.build()
}
