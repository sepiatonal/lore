pub mod engine;
pub mod camera;

use engine::*;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use pollster::block_on;

pub enum InputEvent<'a> {
    Keyboard(&'a KeyboardInput),
    Mouse(&'a ElementState, &'a MouseButton),
    MouseLocation(f64, f64),
}

pub fn run<T: 'static>(
    setup: fn(&mut RenderingInstance) -> T,
    update: fn(&mut RenderingInstance, &mut T) -> (),
    input: fn(&mut RenderingInstance, &mut T, InputEvent) -> (),
) -> T {
    // Initialize logging for WGPU-related errors
    env_logger::init();
    // Create window that will be rendered to
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_inner_size(winit::dpi::PhysicalSize::new(640, 480)).build(&event_loop).unwrap();
    // This instance will be given to the event_loop, will not be kept
    let mut rendering_instance = block_on(RenderingInstance::new(&window));

    let mut state = setup(&mut rendering_instance);

    // Run the main program loop
    event_loop.run(move |event, window_target, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id: _,
            } => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        rendering_instance.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so w have to dereference it twice
                        rendering_instance.resize(**new_inner_size);
                    }
                    WindowEvent::KeyboardInput {
                        input: key_input,
                        ..
                    } => {
                        input(&mut rendering_instance, &mut state, InputEvent::Keyboard(key_input));
                    },
                    WindowEvent::MouseInput {
                        state: button_state,
                        button,
                        ..
                    } => {
                        input(&mut rendering_instance, &mut state, InputEvent::Mouse(button_state, button));
                    },
                    WindowEvent::CursorMoved {
                        position,
                        ..
                    } => {
                        input(&mut rendering_instance, &mut state, InputEvent::MouseLocation(position.x, position.y));
                    }
                    _ => {}
                }
            },
            Event::RedrawRequested(_) => {
                rendering_instance.update();
                update(&mut rendering_instance, &mut state);
                match rendering_instance.draw() {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost) => rendering_instance.refresh_surface_configuration(),
                    // TODO log error
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(wgpu::SurfaceError::Timeout) => {},
                    Err(e) => eprintln!("{:?}", e)
                }
            },
            Event::RedrawEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}