use kardashev_protocol::assets::{
    MeshData,
    PrimitiveTopology,
    Vertex,
    WindingOrder,
};

#[derive(Clone, Debug)]
pub struct ShapeBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    winding_order: WindingOrder,
}

impl Default for ShapeBuilder {
    fn default() -> Self {
        Self::with_capacity(0, 0)
    }
}

impl ShapeBuilder {
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

    pub fn add_tri(&mut self, tri: [u16; 3]) {
        self.indices.extend(tri);
    }

    pub fn add_face(&mut self, vertices: impl AsRef<[u16]>) {
        let vertices = vertices.as_ref();
        for tri in Triangulate::new(vertices.len()) {
            self.add_tri(tri.map(|i| vertices[i]));
        }
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
}

#[derive(Clone, Copy, Debug)]
pub struct Triangulate {
    head: usize,
    left: usize,
    right: usize,
    alternate: bool,
}

impl Triangulate {
    pub fn new(num_vertices: usize) -> Self {
        assert!(num_vertices >= 3);

        Self {
            head: 0,
            left: 1,
            right: num_vertices - 1,
            alternate: true,
        }
    }
}

impl Iterator for Triangulate {
    type Item = [usize; 3];

    fn next(&mut self) -> Option<Self::Item> {
        if self.right > self.left {
            let tri = [self.head, self.left, self.right];
            if self.alternate {
                self.head = self.right;
                self.right -= 1;
            }
            else {
                self.head = self.left;
                self.left += 1;
            };
            self.alternate = !self.alternate;
            Some(tri)
        }
        else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.right - self.left;
        (n, Some(n))
    }
}

impl ExactSizeIterator for Triangulate {}

#[cfg(test)]
mod tests {
    use crate::graphics::mesh::shape::builder::Triangulate;

    fn triangulate(n: usize) -> Vec<[usize; 3]> {
        Triangulate::new(n).collect()
    }

    #[test]
    fn it_triangulates_a_triangle() {
        assert_eq!(triangulate(3), vec![[0, 1, 2]])
    }

    #[test]
    fn it_triangulates_a_pentagon() {
        assert_eq!(triangulate(5), vec![[0, 1, 4], [4, 1, 3], [1, 2, 3],])
    }

    #[test]
    fn it_triangulates_a_hexagon() {
        assert_eq!(
            triangulate(6),
            vec![[0, 1, 5], [5, 1, 4], [1, 2, 4], [4, 2, 3],]
        )
    }
}
