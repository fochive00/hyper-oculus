use super::Vertex3 as Vertex;

#[allow(dead_code)]
pub struct Triangle {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    transform: [[f32; 4]; 4],
}

#[allow(dead_code)]
impl Triangle {
    pub fn new() -> Self {
        Self {
            vertices: vec![
                Vertex{ pos: [ 0.0, -0.5, 0.0], color: [1.0, 0.0, 0.0]},
                Vertex{ pos: [-0.5,  0.5, 0.0], color: [0.0, 1.0, 0.0]},
                Vertex{ pos: [ 0.5,  0.5, 0.0], color: [0.0, 0.0, 1.0]},
            ],
            indices: vec![
                0, 1, 2,
            ],
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ]
        }
    }

    pub fn vertices(&self) -> Vec<Vertex> {
        self.vertices.clone()
    }

    pub fn indices(&self) -> Vec<u16> {
        self.indices.clone()
    }

    pub fn transform(&self) -> [[f32; 4]; 4] {
        self.transform.clone()
    }
}