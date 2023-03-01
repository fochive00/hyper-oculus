
mod core;
mod buffers;
mod pipelines;
mod helpers;
mod entities;
mod cameras;
mod utils;
mod config;
mod app;

use utils::FPScalculator;
use cameras::Camera;

use async_std::task;
use std::sync::{Arc, Mutex};
use std::time;

use winit::{
    event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop}
};

fn main() {
    let event_loop = EventLoop::new();

    let mut application = app::App::new(&event_loop);

    let fps_calculator = Arc::new(Mutex::new(FPScalculator::new()));
    let fps_calculator_clone = Arc::clone(&fps_calculator);
    
    task::spawn(async move {
        loop {
            let fps = fps_calculator_clone.lock().unwrap().fps();
            println!("fps: {}", fps);
    
            let one_second = time::Duration::from_secs(2);
            task::sleep(one_second).await;
        }
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        application.handle_event(&event);

        if let Event::RedrawEventsCleared = event {
            fps_calculator.lock().unwrap().count_one_frame();
        }

        if let Event::WindowEvent { event, .. } = event {
            *control_flow = match event {
                WindowEvent::CloseRequested => ControlFlow::Exit,
                WindowEvent::KeyboardInput { 
                    input: KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), ..},
                    ..
                } => ControlFlow::Exit,
                _ => ControlFlow::Poll,
            }
        }
    });
}
