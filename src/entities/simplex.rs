use super::Vertex;

// use std::time::Instant;
// use chrono::
extern crate nalgebra as na;

pub struct Simplex {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    transform: na::Matrix5<f32>,
}

impl Simplex {
    pub fn new() -> Self {
        let vertices = vec![
            Vertex { pos: [-0.5, -0.5, -0.5, -0.5], color: [1.0, 0.0, 0.0] },
            Vertex { pos: [ 0.5, -0.5,  0.5, -0.5], color: [0.0, 1.0, 0.0] },
            Vertex { pos: [-0.5,  0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0] },
            Vertex { pos: [-0.5,  0.5, -0.5,  0.5], color: [1.0, 1.0, 1.0] },
            Vertex { pos: [ 0.5, -0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] },
            // Vertex { pos: [-0.5, -0.5, -0.5, 1.0], color: [1.0, 0.0, 0.0] },
            // Vertex { pos: [-0.5, -0.5, -0.5, 1.0], color: [0.0, 1.0, 0.0] },
            // Vertex { pos: [-0.5, -0.5,  0.5, 1.0], color: [0.0, 0.0, 1.0] },
            // Vertex { pos: [-0.5, -0.5,  0.5, 1.0], color: [1.0, 1.0, 1.0] },
            // Vertex { pos: [-0.5,  0.5, -0.5, 1.0], color: [1.0, 0.0, 0.0] },
        ];

        let indices = vec![
            0, 1, 2,
            0, 1, 3,
            0, 2, 3,
            1, 2, 3,
            
            0, 1, 4,
            0, 2, 4,
            0, 3, 4,
            1, 2, 4,
            1, 3, 4,
            2, 3, 4,
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