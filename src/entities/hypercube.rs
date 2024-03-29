use super::Vertex;

// use std::time::Instant;
// use chrono::
extern crate nalgebra as na;

pub struct Hypercube {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    transform: na::Matrix5<f32>,
}

impl Hypercube {
    pub fn new() -> Self {
        let vertices = vec![
            Vertex { pos: [-0.5, -0.5, -0.5, -0.5], color: [1.0, 0.0, 0.0] },
            Vertex { pos: [-0.5, -0.5, -0.5,  0.5], color: [0.0, 1.0, 0.0] },
            Vertex { pos: [-0.5, -0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0] },
            Vertex { pos: [-0.5, -0.5,  0.5,  0.5], color: [1.0, 1.0, 1.0] },
            Vertex { pos: [-0.5,  0.5, -0.5, -0.5], color: [1.0, 0.0, 0.0] },
            Vertex { pos: [-0.5,  0.5, -0.5,  0.5], color: [0.0, 1.0, 0.0] },
            Vertex { pos: [-0.5,  0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0] },
            Vertex { pos: [-0.5,  0.5,  0.5,  0.5], color: [1.0, 1.0, 1.0] },
            Vertex { pos: [ 0.5, -0.5, -0.5, -0.5], color: [1.0, 0.0, 0.0] },
            Vertex { pos: [ 0.5, -0.5, -0.5,  0.5], color: [0.0, 1.0, 0.0] },
            Vertex { pos: [ 0.5, -0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0] },
            Vertex { pos: [ 0.5, -0.5,  0.5,  0.5], color: [1.0, 1.0, 1.0] },
            Vertex { pos: [ 0.5,  0.5, -0.5, -0.5], color: [1.0, 0.0, 0.0] },
            Vertex { pos: [ 0.5,  0.5, -0.5,  0.5], color: [0.0, 1.0, 0.0] },
            Vertex { pos: [ 0.5,  0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0] },
            Vertex { pos: [ 0.5,  0.5,  0.5,  0.5], color: [1.0, 1.0, 1.0] },
        ];

        let indices = vec![
            // z, w
            // 0, 1, 2, 3
            0, 1, 2,        1, 2, 3, 
            // 4, 5, 6, 7
            4, 5, 6,        5, 6, 7,
            // 8, 9, 10, 11
            8, 9, 10,       9, 10, 11,
            // 12, 13, 14, 15
            12, 13, 14,     13, 14, 15,
            
            // y, w
            // 0, 1, 4, 5
            0, 1, 4,        1, 4, 5,
            // 2, 3, 6, 7
            2, 3, 6,        3, 6, 7,
            // 8, 9, 12, 13
            8, 9, 12,       9, 12, 13,
            // 10, 11, 14, 15
            10, 11, 14,     11, 14, 15,

            // x, w
            // 0, 1, 8, 9
            0, 1, 8,        1, 8, 9,
            // 2, 3, 10, 11
            2, 3, 10,       3, 10, 11,
            // 4, 5, 12, 13
            4, 5, 12,       5, 12, 13,
            // 6, 7, 14, 15
            6, 7, 14,       7, 14, 15,

            // y, z
            // 0, 2, 4, 6
            0, 2, 4,        2, 4, 6,
            // 1, 3, 5, 7
            1, 3, 5,        3, 5, 7,
            // 8, 10, 12, 14
            8, 10, 12,      10, 12, 14,
            // 9, 11, 13, 15
            9, 11, 13,      11, 13, 15,

            // x, z
            // 0, 2, 8, 10
            0, 2, 8,        2, 8, 10,
            // 1, 3, 9, 11
            1, 3, 9,        3, 9, 11,
            // 4, 6, 12, 14
            4, 6, 12,       6, 12, 14,
            // 5, 7, 13, 15
            5, 7, 13,       7, 13, 15,

            // x, y
            // 0, 4, 8, 12
            0, 4, 8,        4, 8, 12,
            // 1, 5, 9, 13
            1, 5, 9,        5, 9, 13,
            // 2, 6, 10, 14
            2, 6, 10,       6, 10, 14,
            // 3, 7, 11, 15
            3, 7, 11,       7, 11, 15,
        ];

        let transform = na::Matrix5::identity();

        Self {
            vertices,
            indices,
            transform,
        }
    }

    pub fn vertices(&self) -> Vec<Vertex> {
        self.vertices.clone()
    }

    pub fn indices(&self) -> Vec<u16> {
        self.indices.clone()
    }

    pub fn transform(&self) -> na::Matrix5<f32> {
        self.transform.clone()
    }
}