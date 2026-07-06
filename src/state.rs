use crate::renderer::Renderer;
use std::sync::Arc;
use winit::window::Window;

pub(crate) struct AppState {
    pub(crate) window: Arc<Window>,
    pub(crate) renderer: Renderer,
}

impl AppState {
    pub(crate) fn new(window: Arc<Window>, renderer: Renderer) -> Self {
        Self { window, renderer }
    }
}
