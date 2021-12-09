
mod camera_proj3;
pub use camera_proj3::CameraProj3;

mod camera_proj4;
pub use camera_proj4::CameraProj4;
pub use camera_proj4::UniformBufferObject;

mod math;

extern crate nalgebra as na;

pub trait Camera {
    type Transform;
    fn transform(&self) -> Self::Transform;
    fn update_view(&mut self);
    fn handle_event<T>(&mut self, event: &winit::event::Event<T>);
}