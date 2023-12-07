mod color;
mod dag;
mod jit;
mod state;
mod ui_primitives;
mod vec2;

use flexi_logger::Logger;
use state::State;
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn main() {
    Logger::try_with_env().unwrap().start().unwrap();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = pollster::block_on(State::new(window));
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run(move |event, window_target| {
            use winit::event::Event::*;
            match event {
                WindowEvent {
                    window_id: _,
                    event,
                } => {
                    use winit::event::WindowEvent::*;
                    match event {
                        CloseRequested => window_target.exit(),
                        Resized(size) => state.resize(size),
                        RedrawRequested => state.render(),
                        _ => {}
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}
