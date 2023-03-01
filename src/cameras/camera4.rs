
use crate::core::math::{self, cross4};
use super::{Camera, Camera3};

use std::{collections::HashMap, hash::Hash};
use std::time::Instant;
extern crate nalgebra as na;

use winit::{
    event::{ButtonId, DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent}
};
use std::f32::consts::PI;

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
    pub cam4_trans: na::Matrix4<f32>,
    pub cam4_col: na::Vector4<f32>,
    pub cam4_row: na::Vector4<f32>,
    pub cam3_trans: na::Matrix4<f32>,
    pub cam4_const: f32,
}

pub struct Camera4 {
    pub camera3: Camera3,

    fovy: f32,
    near: f32,
    far: f32,

    position: na::Point4<f32>,  // the position of the camera
    target: na::Point4<f32>,   // the point we look at

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

impl Camera4 {
    pub fn new() -> Self {
        let camera3 = Camera3::new();

        // let aspect = 1.0;
        let fovy: f32 = PI / 1.5;
        let near: f32 = -1.0;
        let far: f32 = -100.0;
        
        let position = na::Point4::new(0.0, 0.0, 0.0, 4.0);
        // let position = na::Point4::new(0.0, 0.0, 0.0, 4.0);
        let target = na::Point4::origin();

        let w = (target - position).normalize();
        let y = na::Vector4::new(0.0, 1.0, 0.0, 0.0);
        let z = na::Vector4::new(0.0, 0.0, 1.0, 0.0);
        let x = cross4(&y, &z, &w);

        let movement_speed = 1.0;
        let rotation_speed = 0.1;

        let half_width = (near).abs() / 2.0 * (fovy / 2.0).tan();
        let proj = math::ortho4_short(near, far, half_width) * math::perspective4(near, far);
        // let proj = na::Matrix5::identity();

        let view = math::view4(&position, &x, &y, &z, &w);

        let time = Instant::now();

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
            camera3,
            fovy,
            near,
            far,
            position,
            target,
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
        let cam4_col = transform.fixed_slice::<4,1>(0,4);
        let cam4_row = transform.fixed_slice::<1,4>(4,0).transpose();
        let cam4_trans = transform.fixed_slice::<4,4>(0,0);
        let cam3_trans = self.camera3.transform();
        let cam4_const = transform[(4, 4)];
        UniformBufferObject {
            cam4_trans: cam4_trans.into(),
            cam4_col: cam4_col.into(),
            cam4_row: cam4_row.into(),
            cam3_trans,
            cam4_const,
        }
    }

    pub fn position(&self) -> na::Point4<f32> {
        return self.position
    }

    pub fn set_position(&mut self, pos: na::Point4<f32>) {
        self.position = pos;
    }

    pub fn w(&self) -> na::Vector4<f32> {
        return self.w
    }

    pub fn x(&self) -> na::Vector4<f32> {
        return self.x
    }

    pub fn y(&self) -> na::Vector4<f32> {
        return self.y
    }

    pub fn z(&self) -> na::Vector4<f32> {
        return self.z
    }
}

impl Camera for Camera4 {
    type Transform = na::Matrix5<f32>;

    fn transform(&self) -> Self::Transform {
        self.proj * self.view
        // na::Matrix5::<f32>::identity()
    }

    fn update_view(&mut self) {
        self.camera3.update_view();

        let dt = self.time.elapsed().as_secs_f32();
        self.time = Instant::now();

        // println!("{:?}", self.actions);
        let move_direction = 
            self.actions[&Action::X] * self.x +
            self.actions[&Action::Y] * self.y +
            self.actions[&Action::Z] * self.z +
            self.actions[&Action::W] * self.w;

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
        self.view = math::view4(&self.position, &self.x, &self.y, &self.z, &self.w);
    }

    fn handle_event<T>(&mut self, event: &Event<T>) {
        self.camera3.handle_event(event);

        match event {
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::KeyboardInput {
                    input: KeyboardInput { virtual_keycode: Some(key_code), state, .. },
                    ..
                } = event {
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
                        VirtualKeyCode::Y => *self.actions.get_mut(&Action::X).unwrap() += factor,
                        VirtualKeyCode::H => *self.actions.get_mut(&Action::X).unwrap() -= factor,
                        VirtualKeyCode::U => *self.actions.get_mut(&Action::Y).unwrap() += factor,
                        VirtualKeyCode::J => *self.actions.get_mut(&Action::Y).unwrap() -= factor,
                        VirtualKeyCode::I => *self.actions.get_mut(&Action::Z).unwrap() += factor,
                        VirtualKeyCode::K => *self.actions.get_mut(&Action::Z).unwrap() -= factor,
                        VirtualKeyCode::O => *self.actions.get_mut(&Action::W).unwrap() += factor,
                        VirtualKeyCode::L => *self.actions.get_mut(&Action::W).unwrap() -= factor,
                        _ => ()
                    }
                }
            }

            Event::DeviceEvent { event, .. } => {
                match event {
                    DeviceEvent::MouseMotion { delta } => {
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