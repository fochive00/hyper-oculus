
mod camera3;
pub use camera3::Camera3;

mod camera4;
pub use camera4::Camera4;
pub use camera4::UniformBufferObject;

extern crate nalgebra as na;

pub trait Camera {
    type Transform;
    fn transform(&self) -> Self::Transform;
    fn update_view(&mut self);
    fn handle_event<T>(&mut self, event: &winit::event::Event<T>);
}