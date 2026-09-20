mod app;
mod camera;
mod renderer;
mod scene;
mod ui;

use crate::app::{App, UserEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Warn).expect("Failed to initialize logger");
    }

    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    #[allow(unused_mut)]
    let mut app = App::new(event_loop.create_proxy());

    #[cfg(not(target_arch = "wasm32"))]
    event_loop.run_app(&mut app).expect("Failed to run app");

    // `run_app` does not return on the web; `spawn_app` hands control back to the browser
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(app);
    }
}
