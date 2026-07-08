mod input;
mod state;
mod stats;

pub(crate) use input::*;
pub(crate) use state::*;
pub(crate) use stats::*;

use crate::camera::{Camera, CameraController, CameraDescriptor};
use crate::renderer::{Gpu, RenderSettings, Renderer};
use crate::ui::Gui;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{CursorGrabMode, Window, WindowId};

const APP_NAME: &str = "Oxygen";

pub(crate) struct App {
    app_state: Option<AppState>,

    camera_controller: CameraController,
    input_state: InputState,

    viewport_rect: egui::Rect,

    render_settings: RenderSettings,

    stats: FrameStats,
    last_frame_time: Option<Instant>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            app_state: None,
            camera_controller: CameraController::default(),
            input_state: InputState::default(),
            viewport_rect: egui::Rect::NOTHING,
            render_settings: RenderSettings::default(),
            stats: FrameStats::default(),
            last_frame_time: None,
        }
    }
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
            yaw: 0.0,
            pitch: 0.0,
            up: glam::Vec3::Y,
            aspect: gpu.config.width as f32 / gpu.config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        });

        let renderer = Renderer::new(&camera, &gpu, &mut gui, &self.render_settings, &mut self.stats);

        self.app_state = Some(AppState::new(window, camera, gpu, renderer, gui));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let Some(app_state) = &mut self.app_state else { return };

        let response = app_state.gui.on_window_event(&app_state.window, &event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::MouseInput {
                state,
                button: button @ MouseButton::Right,
                ..
            } => {
                if state == ElementState::Pressed {
                    let over_viewport = app_state.gui.pointer_pos().is_some_and(|p| self.viewport_rect.contains(p));
                    if over_viewport {
                        app_state.window.set_cursor_visible(false);
                        let _ = app_state.window.set_cursor_grab(CursorGrabMode::Locked);
                        self.input_state.mouse_press(button);
                    }
                } else {
                    app_state.window.set_cursor_visible(true);
                    let _ = app_state.window.set_cursor_grab(CursorGrabMode::None);
                    self.input_state.mouse_release(button);
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = self.last_frame_time.map_or(0.0, |last| (now - last).as_secs_f32());
                self.last_frame_time = Some(now);

                // Min to avoid garbage values while focus is lost
                self.stats.set_time(dt.min(1.0));

                let displacement = self.camera_controller.compute(&mut self.input_state, dt.min(0.1));
                app_state.camera.displace(displacement);
                self.viewport_rect = app_state.renderer.render(
                    &app_state.window,
                    &mut app_state.camera,
                    &app_state.gpu,
                    &mut app_state.gui,
                    &mut self.render_settings,
                    &self.stats,
                );
            }
            WindowEvent::Resized(size) => {
                // This only needs to resize the surface as the viewport texture size is handled
                // in the renderer
                app_state.gpu.resize(size);
            }
            WindowEvent::Focused(false) => {
                self.input_state.clear_mouse();
                self.input_state.clear_keys();
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
                            self.input_state.key_press(key_code);
                        }
                    }
                    ElementState::Released => {
                        self.input_state.key_release(key_code);
                    }
                }
            }
            _ => {}
        };
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.input_state.add_mouse_pos_delta(delta.0 as f32, delta.1 as f32);
            }
            DeviceEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
            } => {
                self.input_state.add_mouse_scroll_delta(y);
            }
            DeviceEvent::Key(_) => {}
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = self.app_state.as_ref() {
            state.window.request_redraw();
        }
    }
}
