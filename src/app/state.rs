use crate::camera::Camera;
use crate::renderer::{Gpu, Renderer, Viewport};
use crate::ui::Gui;
use std::sync::Arc;
use winit::window::Window;

pub(crate) struct AppState {
    pub(crate) window: Arc<Window>,
    pub(crate) camera: Camera,
    pub(crate) gpu: Gpu,
    pub(crate) renderer: Renderer,
    pub(crate) gui: Gui,
    pub(crate) viewport: Viewport,
}

impl AppState {
    pub(crate) fn new(window: Arc<Window>, camera: Camera, gpu: Gpu, renderer: Renderer, gui: Gui, viewport: Viewport) -> Self {
        Self {
            window,
            camera,
            gpu,
            renderer,
            gui,
            viewport,
        }
    }
}
