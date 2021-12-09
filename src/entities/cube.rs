
use super::Vertex3 as Vertex;

use std::time::Instant;
// use chrono::
extern crate nalgebra as na;
// use na::{Matrix4, Rotation3};

#[allow(dead_code)]
pub struct Cube {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    transform: na::Matrix4<f32>,
    time: Option<Instant>,
}

#[allow(dead_code)]
impl Cube {
    pub fn new() -> Self {
        Self {
            vertices: vec![
                Vertex{ pos: [-0.5, -0.5, -0.5], color: [1.0, 0.0, 0.0]},
                Vertex{ pos: [-0.5, -0.5,  0.5], color: [0.0, 1.0, 0.0]},
                Vertex{ pos: [-0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0]},
                Vertex{ pos: [-0.5,  0.5,  0.5], color: [1.0, 1.0, 1.0]},
                Vertex{ pos: [ 0.5, -0.5, -0.5], color: [1.0, 0.0, 0.0]},
                Vertex{ pos: [ 0.5, -0.5,  0.5], color: [0.0, 1.0, 0.0]},
                Vertex{ pos: [ 0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0]},
                Vertex{ pos: [ 0.5,  0.5,  0.5], color: [1.0, 1.0, 1.0]}
            ],
            indices: vec![
                // left
                1, 0, 2,    2, 3, 1,
                // right = flip (left + 4) 
                4, 5, 6,    7, 6, 5,

                // bottom
                5, 4, 0,    0, 1, 5,
                // top = flip (bottom + 2)
                6, 7, 2,    3, 2, 7,

                // back
                2, 0, 4,    4, 6, 2,
                // front = flip (back + 1)
                1, 3, 5,    7, 5, 3
            ],
            transform: na::Matrix4::identity(),
            time: None,
        }
    }

    pub fn vertices(&self) -> Vec<Vertex> {
        self.vertices.clone()
    }

    pub fn indices(&self) -> Vec<u16> {
        self.indices.clone()
    }

    pub fn transform(&self) -> na::Matrix4<f32> {
        // self.update();
        self.transform
    }

    pub fn update(&mut self) {
        match self.time {
            None => self.time = Some(Instant::now()),
            Some(t) => {
                let elapsed = t.elapsed().as_secs_f32();
                
                self.transform = na::Matrix4::new_rotation_wrt_point(na::Vector3::x() * elapsed * 1.0, na::OPoint::origin());
            }
        };
    }
}