mod input;
mod state;
mod stats;

pub(crate) use input::*;
pub(crate) use state::*;
pub(crate) use stats::*;
use std::collections::VecDeque;

use crate::camera::{Camera, CameraController};
use crate::renderer::{Gpu, RenderSettings, Renderer, Viewport};
use crate::scene::{Model, Scene, load};
use crate::ui::{EditorCommand, Gui, editor};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Instant;
use wgpu::CurrentSurfaceTexture;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{CursorGrabMode, Window, WindowId};

const APP_NAME: &str = "Oxygen";

const SCROLL_PIXELS_PER_LINE: f32 = 50.0;
const PINCH_LINES_PER_MAGNIFICATION: f32 = 40.0;

pub(crate) struct App {
    app_state: Option<AppState>,

    scene: Scene,

    camera_controller: CameraController,
    input_state: InputState,

    viewport_rect: egui::Rect,

    render_settings: RenderSettings,

    stats: FrameStats,
    last_frame_time: Option<Instant>,

    editor_commands: VecDeque<EditorCommand>,

    tx: mpsc::Sender<Model>,
    rx: mpsc::Receiver<Model>,
}

impl Default for App {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            app_state: None,
            scene: Scene::default(),
            camera_controller: CameraController::default(),
            input_state: InputState::default(),
            viewport_rect: egui::Rect::NOTHING,
            render_settings: RenderSettings::default(),
            stats: FrameStats::default(),
            last_frame_time: None,
            editor_commands: VecDeque::new(),
            tx,
            rx,
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
        let viewport = Viewport::new(&gpu.device, &mut gui, gpu.config.width, gpu.config.height);
        let renderer = Renderer::new(&self.scene.camera, &self.scene.light, &gpu, &self.render_settings);

        self.scene.camera.update_aspect_ratio(viewport.width as f32, viewport.height as f32);

        self.app_state = Some(AppState::new(window, gpu, renderer, gui, viewport));
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
            WindowEvent::PinchGesture { delta, .. } => {
                let over_viewport = app_state.gui.pointer_pos().is_some_and(|p| self.viewport_rect.contains(p));
                if over_viewport && delta.is_finite() {
                    self.input_state.add_mouse_scroll_delta(delta as f32 * PINCH_LINES_PER_MAGNIFICATION);
                }
            }
            WindowEvent::RedrawRequested => {
                while let Some(command) = self.editor_commands.pop_front() {
                    handle(event_loop, &mut self.scene.camera, self.viewport_rect, command, self.tx.clone());
                }

                while let Ok(model) = self.rx.try_recv() {
                    app_state.renderer.load(&app_state.gpu, &model);
                    self.scene.model = model;
                }

                let now = Instant::now();
                let dt = self.last_frame_time.map_or(0.0, |last| (now - last).as_secs_f32());
                self.last_frame_time = Some(now);

                // Min to avoid garbage values while focus is lost
                self.stats.set_time(dt.min(1.0));
                self.stats.update(&self.scene.model);

                let displacement = self.camera_controller.compute(&mut self.input_state, dt.min(0.1));
                self.scene.camera.displace(displacement);

                let frame = match app_state.gpu.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(frame) => frame,
                    CurrentSurfaceTexture::Suboptimal(frame) => frame,
                    _ => return,
                };

                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = app_state.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                app_state
                    .renderer
                    .render(&self.scene, &app_state.gpu, &mut encoder, &app_state.viewport, &mut self.render_settings);

                let mut viewport_rect = egui::Rect::NOTHING;
                app_state.gui.render(
                    &app_state.window,
                    &app_state.gpu.device,
                    &app_state.gpu.queue,
                    &mut encoder,
                    &view,
                    |ui| {
                        viewport_rect = editor::build(
                            ui,
                            app_state.viewport.texture_id,
                            &mut self.scene.model,
                            &mut self.scene.light,
                            &mut self.render_settings,
                            &self.stats,
                            &mut self.editor_commands,
                        );
                    },
                );

                if self.viewport_rect.size() != viewport_rect.size() {
                    self.viewport_rect = viewport_rect;

                    app_state.viewport.resize(
                        &app_state.gpu.device,
                        &mut app_state.gui,
                        self.viewport_rect.size().x,
                        self.viewport_rect.size().y,
                    );

                    self.scene.camera.update_aspect_ratio(viewport_rect.size().x, viewport_rect.size().y);
                }

                app_state.gpu.queue.submit(std::iter::once(encoder.finish()));
                frame.present();
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
            DeviceEvent::MouseWheel { delta } => {
                if let Some(app_state) = &mut self.app_state {
                    let over_viewport = app_state.gui.pointer_pos().is_some_and(|p| self.viewport_rect.contains(p));
                    if over_viewport {
                        let lines = match delta {
                            MouseScrollDelta::LineDelta(_, y) => y,
                            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / SCROLL_PIXELS_PER_LINE,
                        };
                        self.input_state.add_mouse_scroll_delta(lines);
                    }
                }
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

fn handle(event_loop: &ActiveEventLoop, camera: &mut Camera, viewport_rect: egui::Rect, cmd: EditorCommand, tx: mpsc::Sender<Model>) {
    match cmd {
        EditorCommand::LoadFile(path) => {
            let path = path.to_string_lossy().to_string();

            thread::spawn(move || {
                let model = load(path);
                tx.send(model).ok();
            });
        }
        EditorCommand::ResetCamera => {
            camera.update_aspect_ratio(viewport_rect.size().x, viewport_rect.size().y);
        }
        EditorCommand::Quit => {
            event_loop.exit();
        }
    }
}
