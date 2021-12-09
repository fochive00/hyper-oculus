
use std::{collections::HashMap, hash::Hash};
use std::time::Instant;

extern crate nalgebra as na;

use winit::{
    event::{ButtonId, DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent}
};
use std::f32::consts::PI;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum Input {
    Button(ButtonId),
    Key(VirtualKeyCode),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Action {
    Forward,
    Right,
    Up,
    Yaw,
    Pitch,
}

#[allow(dead_code)]
pub struct CameraProj3 {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    position: na::Point3<f32>,
    look_direction: na::Vector3<f32>,
    right_direction: na::Vector3<f32>,

    movement_speed: f32,
    rotation_speed: f32,

    flip_y: bool,

    view: na::Matrix4<f32>,
    proj: na::Matrix4<f32>,

    time: Instant,
    input_map: HashMap<Input, ElementState>,
    actions: HashMap<Action, f32>,
}

impl CameraProj3 {
    pub fn new() -> Self {
        let aspect = 16.0 / 9.0;
        let fovy = 3.14 / 4.0;
        let znear = 0.1;
        let zfar = 1000.0;

        let position = na::Point3::new(2.0, 2.0, 2.0);
        // let position = na::point!(2.0, 2.0, 2.0);
        let target = na::Point3::origin();
        let look_direction = (target - position).normalize();
        let right_direction = look_direction.cross(&na::Vector3::z()).normalize();
        
        let movement_speed = 5.0;
        let rotation_speed = 0.1;

        let flip_y = true;

        let view = na::Isometry3::look_at_rh(
            &position, 
            &target,
            &right_direction.cross(&look_direction).normalize()
        ).to_homogeneous();

        let proj = na::Perspective3::new(
            aspect,
            fovy,
            znear,
            zfar
        ).to_homogeneous();

        let time = Instant::now();

        // println!("projective: {:?}", proj);
        let input_map = HashMap::new();

        let actions = {
            let mut actions = HashMap::new();
            actions.insert(Action::Forward, 0.0);
            actions.insert(Action::Right,   0.0);
            actions.insert(Action::Up,      0.0);
            actions.insert(Action::Yaw,     0.0);
            actions.insert(Action::Pitch,   0.0);

            actions
        };

        Self {
            aspect,
            fovy,
            znear,
            zfar,
            position,
            look_direction,
            right_direction,
            movement_speed,
            rotation_speed,
            flip_y,
            view,
            proj,
            time,
            input_map,
            actions
        }
    }


    fn up_direction(&self) -> na::Vector3<f32> {
        self.right_direction.cross(&self.look_direction)
    }

    // Use DeviceEvent to detect KeyboardInput, not work.
    #[allow(dead_code)]
    pub fn handle_device_event(&mut self, event: &winit::event::DeviceEvent) {
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                *self.actions.get_mut(&Action::Yaw).unwrap() += delta.0 as f32;
                *self.actions.get_mut(&Action::Pitch).unwrap() += delta.1 as f32;
            }
            DeviceEvent::Button { state, button } => {
                println!("button: {:?}", button);
                self.input_map.insert(Input::Button(*button), *state);
            }
            winit::event::DeviceEvent::Key(key) => {
                println!("key: {:?}", key);
                if let Some(key_code) = key.virtual_keycode {
                    let prev_state = self.input_map.insert(Input::Key(key_code), key.state);

                    let factor = if prev_state == Some(key.state) {
                        0.0 // repeat
                    } else {
                        match key.state {
                            ElementState::Pressed => 1.0,
                            ElementState::Released => -1.0,
                        }
                    };

                    match key_code {
                        VirtualKeyCode::W => *self.actions.get_mut(&Action::Forward).unwrap() += factor,
                        VirtualKeyCode::S => *self.actions.get_mut(&Action::Forward).unwrap() -= factor,
                        VirtualKeyCode::D => *self.actions.get_mut(&Action::Right).unwrap() += factor,
                        VirtualKeyCode::A => *self.actions.get_mut(&Action::Right).unwrap() -= factor,
                        VirtualKeyCode::C => *self.actions.get_mut(&Action::Up).unwrap() += factor,
                        VirtualKeyCode::V => *self.actions.get_mut(&Action::Up).unwrap() -= factor,
                        _ => ()
                    }
                }
            }
            _ => (),
        }
    }
}

impl super::Camera for CameraProj3 {
    type Transform = na::Matrix4<f32>;
    
    fn transform(&self) -> Self::Transform {
        self.proj * self.view
    }

    fn update_view(&mut self) {
        let dt = self.time.elapsed().as_secs_f32();
        self.time = Instant::now();

        let mut flip_y = 1.0;
        if self.flip_y {
            flip_y = -1.0;
        }
        // println!("{:?}", self.actions);
        let move_direction = 
            self.actions[&Action::Forward] * self.look_direction +
            self.actions[&Action::Right] * self.right_direction +
            self.actions[&Action::Up] * self.up_direction() * flip_y;

        // rotation
        let dx = self.actions[&Action::Yaw] * self.rotation_speed / 180.0 * PI;
        let dy = self.actions[&Action::Pitch] * self.rotation_speed / 180.0 * PI * flip_y;

        *self.actions.get_mut(&Action::Yaw).unwrap() = 0.0;
        *self.actions.get_mut(&Action::Pitch).unwrap() = 0.0;

        let axis = na::Unit::new_normalize(self.look_direction.cross(&self.right_direction));
        let rot_quat1 = na::UnitQuaternion::from_axis_angle(&axis, dx);
        let axis = na::Unit::new_normalize(-1.0 * self.right_direction);
        let rot_quat2 = na::UnitQuaternion::from_axis_angle(&axis, dy);
        let rot_quat = rot_quat1.nlerp(&rot_quat2, 0.5);
        self.look_direction = rot_quat * self.look_direction;
        self.right_direction = rot_quat * self.right_direction;

        // transform
        self.position += move_direction * self.movement_speed * dt;

        // update view matrix
        self.view = {
            let view = na::Isometry3::look_at_rh(
                &self.position, 
                &(self.position + self.look_direction),
                &self.right_direction.cross(&self.look_direction).normalize()
            ).to_homogeneous();

            view
        }
    }

    fn handle_event<T>(&mut self, event: &Event<T>) {
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
                            VirtualKeyCode::W => *self.actions.get_mut(&Action::Forward).unwrap() += factor,
                            VirtualKeyCode::S => *self.actions.get_mut(&Action::Forward).unwrap() -= factor,
                            VirtualKeyCode::D => *self.actions.get_mut(&Action::Right).unwrap() += factor,
                            VirtualKeyCode::A => *self.actions.get_mut(&Action::Right).unwrap() -= factor,
                            VirtualKeyCode::C => *self.actions.get_mut(&Action::Up).unwrap() += factor,
                            VirtualKeyCode::V => *self.actions.get_mut(&Action::Up).unwrap() -= factor,
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