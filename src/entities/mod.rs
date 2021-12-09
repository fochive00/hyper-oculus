mod cube;
pub use cube::Cube;

// mod triangle;
// pub use triangle::Triangle;

mod simplex;
pub use simplex::Simplex;

// mod hypercube;
// pub use hypercube::Hypercube;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Vertex3 {
    pub pos: [f32; 3],
    pub color: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Vertex4 {
    pub pos: [f32; 4],
    pub color: [f32; 3],
}