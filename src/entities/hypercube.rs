use super::Vertex4 as Vertex;

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
            // w, x, y, z
            1, 0, 2,    2, 3, 1,
            4, 5, 6,    7, 6, 5,
            5, 4, 0,    0, 1, 5,
            6, 7, 2,    3, 2, 7,
            2, 0, 4,    4, 6, 2,
            1, 3, 5,    7, 5, 3
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