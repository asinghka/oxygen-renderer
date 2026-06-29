use ash::vk;
use ash_bootstrap::{Instance, InstanceBuilder};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    window: Option<Window>,
    instance: Option<Arc<Instance>>,
}

impl App {
    fn init_window(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        let window = event_loop.create_window(Window::default_attributes())?;

        // ash-bootstrap welds the window's surface into the instance, so we build it only once and
        // never on a second `resumed`; safe only because desktop (mac/Windows) fires `resumed` once.
        if self.instance.is_none() {
            let window_handle = window.window_handle()?;
            let display_handle = window.display_handle()?;

            let instance = InstanceBuilder::new(Some((window_handle, display_handle)))
                .app_name("Vulkan Renderer")
                .require_api_version(vk::API_VERSION_1_3)
                .request_validation_layers(cfg!(debug_assertions))
                .use_default_debug_messenger()
                .build()?;
            self.instance = Some(instance);
        }

        self.window = Some(window);

        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self.init_window(event_loop) {
            Ok(_) => {}
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

impl Drop for App {
    fn drop(&mut self) {
        if let Some(instance) = self.instance.take() {
            instance.destroy();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
