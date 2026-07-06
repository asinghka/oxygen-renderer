use crate::gpu::Gpu;
use crate::renderer::Renderer;
use std::sync::Arc;
use winit::window::Window;

pub(crate) struct AppState {
    pub(crate) window: Arc<Window>,
    pub(crate) gpu: Gpu,
    pub(crate) renderer: Renderer,
}

impl AppState {
    pub(crate) fn new(window: Arc<Window>, gpu: Gpu, renderer: Renderer) -> Self {
        Self { window, gpu, renderer }
    }
}
