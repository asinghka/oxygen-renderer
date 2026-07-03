use egui::ViewportId;
use egui_wgpu::RendererOptions;
use egui_winit::EventResponse;
use winit::event::WindowEvent;
use winit::window::Window;

pub(crate) struct Gui {
    pub(crate) context: egui::Context,
    pub(crate) winit_state: egui_winit::State,
    pub(crate) renderer: egui_wgpu::Renderer,
}

impl Gui {
    pub(crate) fn new(window: &Window, device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Self {
        let context = egui::Context::default();
        let winit_state = egui_winit::State::new(
            context.clone(),
            ViewportId::ROOT,
            window,
            None,
            window.theme(),
            Some(device.limits().max_texture_dimension_2d as usize),
        );
        let renderer = egui_wgpu::Renderer::new(device, texture_format, RendererOptions::default());

        Self {
            context,
            winit_state,
            renderer,
        }
    }

    pub(crate) fn on_window_event(&mut self, window: &Window, event: &WindowEvent) -> EventResponse {
        self.winit_state.on_window_event(window, event)
    }
}
