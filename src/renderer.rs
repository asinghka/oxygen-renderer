use egui::load::SizedTexture;
use egui::{Frame, ViewportId, vec2};
use egui_wgpu::RendererOptions;
use pollster::FutureExt;
use std::sync::Arc;
use wgpu::{
    Backends, Color, CurrentSurfaceTexture, Extent3d, Features, FilterMode, LoadOp, Operations, PowerPreference, StoreOp, TextureDimension,
    TextureFormat, TextureUsages,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub(crate) struct Renderer {
    pub(crate) window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,

    egui_context: egui::Context,
    pub(crate) egui_winit_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,

    _viewport_texture: wgpu::Texture,
    viewport_texture_view: wgpu::TextureView,
    viewport_texture_id: egui::TextureId,
}

impl Renderer {
    pub(crate) fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .block_on()
            .expect("Failed to create an adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                required_limits: Default::default(),
                experimental_features: Default::default(),
                memory_hints: Default::default(),
                trace: Default::default(),
            })
            .block_on()
            .expect("Failed to create a device");

        let surface = instance.create_surface(window.clone()).expect("Failed to create surface");
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let egui_context = egui::Context::default();
        let egui_winit_state = egui_winit::State::new(
            egui_context.clone(),
            ViewportId::ROOT,
            window.as_ref(),
            None,
            window.theme(),
            Some(device.limits().max_texture_dimension_2d as usize),
        );
        let mut egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, RendererOptions::default());

        let viewport_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let viewport_texture_view = viewport_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let viewport_texture_id = egui_renderer.register_native_texture(&device, &viewport_texture_view, FilterMode::Linear);

        Self {
            window,
            device,
            queue,
            surface,
            config,
            egui_context,
            egui_winit_state,
            egui_renderer,
            _viewport_texture: viewport_texture,
            viewport_texture_view,
            viewport_texture_id,
        }
    }

    pub(crate) fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => frame,
            CurrentSurfaceTexture::Suboptimal(frame) => frame,
            _ => return,
        };

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let raw_input = self.egui_winit_state.take_egui_input(&self.window);
        let full_output = self.egui_context.run_ui(raw_input, |ui| {
            egui::Panel::bottom("debug-panel").show(ui, |ui| {
                ui.take_available_space();
            });
            egui::Panel::left("scene-tree").show(ui, |ui| {
                ui.take_available_space();
            });
            egui::Panel::right("inspector").show(ui, |ui| {
                ui.take_available_space();
            });
            egui::CentralPanel::default().frame(Frame::NONE).show(ui, |ui| {
                ui.image(SizedTexture::new(
                    self.viewport_texture_id,
                    vec2(self.config.width as f32, self.config.height as f32),
                ));
            });
        });
        self.egui_winit_state.handle_platform_output(&self.window, full_output.platform_output);
        let clipped_primitives = self.egui_context.tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, delta);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: full_output.pixels_per_point,
        };
        self.egui_renderer
            .update_buffers(&self.device, &self.queue, &mut encoder, &clipped_primitives, &screen_descriptor);

        {
            let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.viewport_texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::GREEN),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLUE),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.egui_renderer
                .render(&mut render_pass.forget_lifetime(), &clipped_primitives, &screen_descriptor);

            full_output.textures_delta.free;
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }

    pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }
}
