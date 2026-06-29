use ash_bootstrap::InstanceBuilder;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::{Window, WindowId};

fn init_window(event_loop: &ActiveEventLoop) -> anyhow::Result<Window> {
    let window = event_loop.create_window(Window::default_attributes())?;

    let window_handle = window.window_handle()?;
    let display_handle = window.display_handle()?;

    let _instance = InstanceBuilder::new(Some((window_handle, display_handle)))
        .app_name("Vulkan Renderer")
        .request_validation_layers(true)
        .use_default_debug_messenger()
        .build()?;

    Ok(window)
}

#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match init_window(event_loop) {
            Ok(window) => {
                self.window = Some(window);
            }
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        };
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
