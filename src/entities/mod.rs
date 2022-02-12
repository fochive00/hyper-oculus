mod entity;
pub use entity::Entity;

// mod hypercube;
// pub use hypercube::Hypercube;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 3],
}