use crate::gpu::Gpu;
use crate::input::InputState;
use crate::renderer::Renderer;
use crate::state::AppState;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

const APP_NAME: &str = "Oxygen";

#[derive(Default)]
pub(crate) struct App {
    app_state: Option<AppState>,
    input_state: InputState,
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

        let gpu = Gpu::new(window.clone());
        let renderer = Renderer::new(window.clone(), &gpu);

        self.app_state = Some(AppState::new(window, gpu, renderer));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let Some(app_state) = &mut self.app_state else { return };

        let response = app_state.renderer.gui.on_window_event(&app_state.window, &event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                app_state.renderer.update(&self.input_state, &app_state.gpu);
                app_state.renderer.render(&app_state.window, &app_state.gpu);
            }
            WindowEvent::Resized(size) => {
                app_state.gpu.resize(size);
            }
            WindowEvent::Focused(false) => {
                self.input_state.clear();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        ..
                    },
                ..
            } => {
                match state {
                    ElementState::Pressed => {
                        if !response.consumed {
                            self.input_state.press(key_code);
                        }
                    }
                    ElementState::Released => {
                        self.input_state.release(key_code);
                    }
                }
            }
            _ => {}
        };
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = self.app_state.as_ref() {
            state.window.request_redraw();
        }
    }
}
