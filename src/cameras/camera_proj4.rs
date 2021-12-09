
use super::math;
use super::{Camera, CameraProj3};

use std::{collections::HashMap, hash::Hash};
use std::time::Instant;
extern crate nalgebra as na;

use winit::{
    event::{ButtonId, DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent}
};
// use std::f32::consts::PI;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum Input {
    Button(ButtonId),
    Key(VirtualKeyCode),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Action {
    W,
    X,
    Y,
    Z,
    Yaw,
    Pitch,
}

#[derive(Copy, Debug, Clone)]
#[allow(dead_code)]
pub struct UniformBufferObject {
    cam4_trans: na::Matrix4<f32>,
    cam4_col: na::Vector4<f32>,
    cam4_row: na::Vector4<f32>,
    cam3_trans: na::Matrix4<f32>,
    cam4_const: f32,
}

pub struct CameraProj4 {
    camera_proj3: CameraProj3,

    fovy: f32,
    near: f32,
    far: f32,

    position: na::Point4<f32>,

    w: na::Vector4<f32>,
    x: na::Vector4<f32>,
    y: na::Vector4<f32>,
    z: na::Vector4<f32>,

    movement_speed: f32,
    rotation_speed: f32,

    view: na::Matrix5<f32>,
    proj: na::Matrix5<f32>,

    time: Instant,
    input_map: HashMap<Input, ElementState>,
    actions: HashMap<Action, f32>,
}

impl CameraProj4 {
    pub fn new() -> Self {
        let camera_proj3 = CameraProj3::new();

        let fovy = 3.14 / 4.0;
        let near = -100.0;
        let far = 100.0;
        
        // let position = na::Point4::new(20.0, 20.0, 20.0, 20.0);
        let position = na::Point4::new(20.0, 0.0, 0.0, 0.0);

        let target = na::Point4::origin();
        let w_axis = na::Vector4::new(1.0, 0.0, 0.0, 0.0);
        let x_axis = na::Vector4::new(0.0, 1.0, 0.0, 0.0);
        let y_axis = na::Vector4::new(0.0, 0.0, 1.0, 0.0);
        let z_axis = na::Vector4::new(0.0, 0.0, 0.0, 1.0);

        let w = (target - position).normalize();
        let z = math::cross(&w, &x_axis, &y_axis);
        let y = math::cross(&w, &x_axis, &z);
        let x = math::cross(&w, &y, &z);
        println!("w: {:?}", w);
        println!("x: {:?}", x);
        println!("y: {:?}", y);
        println!("z: {:?}", z);

        let movement_speed = 10.0;
        let rotation_speed = 0.1;

        let proj = math::ortho_short(near, far, 100.0);
        // let proj = na::Matrix5::identity();
        let view = math::view(&position, &w, &x, &y, &z);
        println!("view: {:?}", view);

        let time = Instant::now();

        // println!("projective: {:?}", proj);
        let input_map = HashMap::new();

        let actions = {
            let mut actions = HashMap::new();
            actions.insert(Action::W, 0.0);
            actions.insert(Action::X, 0.0);
            actions.insert(Action::Y, 0.0);
            actions.insert(Action::Z, 0.0);
            actions.insert(Action::Yaw,     0.0);
            actions.insert(Action::Pitch,   0.0);

            actions
        };

        Self {
            camera_proj3,
            fovy,
            near,
            far,
            position,
            w,
            x,
            y,
            z,
            movement_speed,
            rotation_speed,
            view,
            proj,
            time,
            input_map,
            actions
        }
    }

    pub fn data(&self, model: &na::Matrix5<f32>) -> UniformBufferObject {
        let transform = self.transform() * model;
        let cam4_col = transform.column(3).remove_row(3);
        let cam4_row = transform.row(3).remove_column(3).transpose();
        let cam4_trans = transform.remove_row(3).remove_column(3);
        let cam3_trans = self.camera_proj3.transform();
        let cam4_const = transform[(4, 4)];
        UniformBufferObject {
            cam4_trans,
            cam4_col,
            cam4_row,
            cam3_trans,
            cam4_const,
        }
    }
}

impl Camera for CameraProj4 {
    type Transform = na::Matrix5<f32>;

    fn transform(&self) -> Self::Transform {
        self.proj * self.view
    }

    fn update_view(&mut self) {
        self.camera_proj3.update_view();

        let dt = self.time.elapsed().as_secs_f32();
        self.time = Instant::now();

        // println!("{:?}", self.actions);
        let move_direction = 
            self.actions[&Action::W] * self.w +
            self.actions[&Action::X] * self.x +
            self.actions[&Action::Y] * self.y +
            self.actions[&Action::Z] * self.z;

        // rotation
        // let dx = self.actions[&Action::Yaw] * self.rotation_speed / 180.0 * PI;
        // let dy = self.actions[&Action::Pitch] * self.rotation_speed / 180.0 * PI * flip_y;

        // *self.actions.get_mut(&Action::Yaw).unwrap() = 0.0;
        // *self.actions.get_mut(&Action::Pitch).unwrap() = 0.0;

        // let axis = na::Unit::new_normalize(self.look_direction.cross(&self.right_direction));
        // let rot_quat1 = na::UnitQuaternion::from_axis_angle(&axis, dx);
        // let axis = na::Unit::new_normalize(-1.0 * self.right_direction);
        // let rot_quat2 = na::UnitQuaternion::from_axis_angle(&axis, dy);
        // let rot_quat = rot_quat1.nlerp(&rot_quat2, 0.5);
        // self.look_direction = rot_quat * self.look_direction;
        // self.right_direction = rot_quat * self.right_direction;

        // transform
        self.position += move_direction * self.movement_speed * dt;

        // update view matrix
        self.view = math::view(&self.position, &self.w, &self.x, &self.y, &self.z);
    }

    fn handle_event<T>(&mut self, event: &Event<T>) {
        self.camera_proj3.handle_event(event);

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(key_code),
                                state,
                                ..
                            },
                        ..
                    } => {
                        let prev_state = self.input_map.insert(Input::Key(*key_code), *state);

                        let factor = if prev_state == Some(*state) {
                            0.0 // repeat
                        } else {
                            match state {
                                ElementState::Pressed => 1.0,
                                ElementState::Released => -1.0,
                            }
                        };

                        match key_code {
                            VirtualKeyCode::Y => *self.actions.get_mut(&Action::W).unwrap() += factor,
                            VirtualKeyCode::H => *self.actions.get_mut(&Action::W).unwrap() -= factor,
                            VirtualKeyCode::U => *self.actions.get_mut(&Action::X).unwrap() += factor,
                            VirtualKeyCode::J => *self.actions.get_mut(&Action::X).unwrap() -= factor,
                            VirtualKeyCode::I => *self.actions.get_mut(&Action::Y).unwrap() += factor,
                            VirtualKeyCode::K => *self.actions.get_mut(&Action::Y).unwrap() -= factor,
                            VirtualKeyCode::O => *self.actions.get_mut(&Action::Z).unwrap() += factor,
                            VirtualKeyCode::L => *self.actions.get_mut(&Action::Z).unwrap() -= factor,
                            _ => ()
                        }
                    },
                    _ => ()
                }
            }

            Event::DeviceEvent { event, .. } => {
                match event {
                    winit::event::DeviceEvent::MouseMotion { delta } => {
                        *self.actions.get_mut(&Action::Yaw).unwrap() += delta.0 as f32;
                        *self.actions.get_mut(&Action::Pitch).unwrap() += delta.1 as f32;
                    }
                    DeviceEvent::Button { state, button } => {
                        self.input_map.insert(Input::Button(*button), *state);
                    }
                    _ => ()
                }
            }
            _ => ()
        }
    }
}