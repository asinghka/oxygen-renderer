use crate::input::InputHandler;
use crate::renderer::Renderer;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

const APP_NAME: &str = "Oxygen";

#[derive(Default)]
pub(crate) struct App {
    renderer: Option<Renderer>,
    input_handler: InputHandler,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(APP_NAME)
                        .with_inner_size(winit::dpi::PhysicalSize::new(1600, 900)),
                )
                .expect("Failed to create window"),
        );

        self.renderer = Some(Renderer::new(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let Some(renderer) = &mut self.renderer else { return };

        let response = renderer.gui.on_window_event(&renderer.window, &event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                renderer.update(&self.input_handler);
                renderer.render();
            }
            WindowEvent::Resized(size) => {
                renderer.resize(size);
            }
            WindowEvent::Focused(false) => {
                self.input_handler.clear();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                match event {
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        ..
                    } => {
                        match state {
                            ElementState::Pressed => {
                                if !response.consumed {
                                    self.input_handler.key_pressed(key_code);
                                }
                            }
                            ElementState::Released => {
                                self.input_handler.key_released(key_code);
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        };
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = self.renderer.as_ref() {
            state.window.request_redraw();
        }
    }
}
