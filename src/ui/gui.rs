use egui::ViewportId;
use egui_wgpu::RendererOptions;
use egui_winit::EventResponse;
use wgpu::{Color, LoadOp, Operations, StoreOp};
use winit::event::WindowEvent;
use winit::window::Window;

pub(crate) struct Gui {
    context: egui::Context,
    winit_state: egui_winit::State,
    pub(crate) renderer: egui_wgpu::Renderer,
}

impl Gui {
    pub(crate) fn new(window: &Window, device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Self {
        let context = egui::Context::default();
        re_ui::apply_style_and_install_loaders(&context);

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

    pub(crate) fn pointer_pos(&self) -> Option<egui::Pos2> {
        self.context.pointer_latest_pos()
    }

    pub(crate) fn pixels_per_point(&self) -> f32 {
        self.context.pixels_per_point()
    }

    pub(crate) fn render(
        &mut self,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        texture_view: &wgpu::TextureView,
        build: impl FnMut(&mut egui::Ui),
    ) {
        let raw_input = self.winit_state.take_egui_input(window);
        let full_output = self.context.run_ui(raw_input, build);
        self.winit_state.handle_platform_output(window, full_output.platform_output);
        let clipped_primitives = self.context.tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, delta);
        }

        let size = window.inner_size();
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: full_output.pixels_per_point,
        };

        self.renderer
            .update_buffers(device, queue, encoder, &clipped_primitives, &screen_descriptor);

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.renderer
                .render(&mut render_pass.forget_lifetime(), &clipped_primitives, &screen_descriptor);

            for id in &full_output.textures_delta.free {
                self.renderer.free_texture(id);
            }
        }
    }
}
