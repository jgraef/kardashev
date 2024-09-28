//pub mod teapot;

use nalgebra::{
    Vector2,
    Vector3,
};
use wgpu::util::DeviceExt;

use crate::graphics::rendering_system::Vertex;

#[derive(Clone, Debug)]
pub struct MeshData {
    pub primitive_topology: PrimitiveTopology,
    pub indices: Vec<usize>,
    pub positions: Vec<Vector3<f32>>,
    pub normals: Vec<Vector3<f32>>,
    pub tex_coords: Vec<Vector2<f32>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
}

#[derive(Debug)]
pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl Mesh {
    pub fn test(device: &wgpu::Device) -> Self {
        const VERTICES: &[Vertex] = &[
            Vertex {
                position: [-0.0868241, 0.49240386, 0.0],
                tex_coords: [0.4131759, 0.99240386],
            }, // A
            Vertex {
                position: [-0.49513406, 0.06958647, 0.0],
                tex_coords: [0.0048659444, 0.56958647],
            }, // B
            Vertex {
                position: [-0.21918549, -0.44939706, 0.0],
                tex_coords: [0.28081453, 0.05060294],
            }, // C
            Vertex {
                position: [0.35966998, -0.3473291, 0.0],
                tex_coords: [0.85967, 0.1526709],
            }, // D
            Vertex {
                position: [0.44147372, 0.2347359, 0.0],
                tex_coords: [0.9414737, 0.7347359],
            }, // E
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: INDICES.len() as u32,
        }
    }
}
