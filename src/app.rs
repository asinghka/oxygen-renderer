use crate::camera::{Camera, CameraController, CameraDescriptor};
use crate::gpu::Gpu;
use crate::gui::Gui;
use crate::input::InputState;
use crate::renderer::Renderer;
use crate::state::AppState;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

const APP_NAME: &str = "Oxygen";

#[derive(Default)]
pub(crate) struct App {
    app_state: Option<AppState>,
    camera_controller: CameraController,
    input_state: InputState,
    last_frame_time: Option<Instant>,
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
        let mut gui = Gui::new(&window, &gpu.device, gpu.config.format);

        let camera = Camera::new(&CameraDescriptor {
            eye: glam::vec3(0.0, 0.0, 2.0),
            target: glam::Vec3::ZERO,
            up: glam::Vec3::Y,
            aspect: gpu.config.width as f32 / gpu.config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        });

        let renderer = Renderer::new(&camera, &gpu, &mut gui);

        self.app_state = Some(AppState::new(window, camera, gpu, renderer, gui));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let Some(app_state) = &mut self.app_state else { return };

        let response = app_state.gui.on_window_event(&app_state.window, &event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = self.last_frame_time.map_or(0.0, |last| (now - last).as_secs_f32()).min(0.1);
                self.last_frame_time = Some(now);

                let displacement = self.camera_controller.compute(&self.input_state, dt);
                app_state.camera.displace(displacement);
                app_state
                    .renderer
                    .render(&app_state.window, &mut app_state.camera, &app_state.gpu, &mut app_state.gui);
            }
            WindowEvent::Resized(size) => {
                // This only needs to resize the surface as the viewport texture size is handled
                // in the renderer
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
