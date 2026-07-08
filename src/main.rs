mod app;
mod camera;
mod editor;
mod gpu;
mod gui;
mod input;
mod model;
mod renderer;
mod settings;
mod state;
mod stats;
mod style;
mod vertex;
mod viewport;

use crate::app::App;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).expect("Failed to run app");
}
