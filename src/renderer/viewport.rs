use crate::ui::Gui;
use wgpu::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

pub(crate) struct Viewport {
    pub(crate) width: u32,
    pub(crate) height: u32,

    // Held only to keep the GPU resource alive
    texture: wgpu::Texture,

    pub(crate) texture_view: wgpu::TextureView,
    pub(crate) texture_id: egui::TextureId,

    // Held only to keep the GPU resource alive
    depth_texture: wgpu::Texture,
    pub(crate) depth_texture_view: wgpu::TextureView,
}

impl Viewport {
    pub(crate) fn new(device: &wgpu::Device, gui: &mut Gui, width: u32, height: u32) -> Self {
        let (texture, texture_view) = Self::create_texture(device, width, height);
        let texture_id = gui.renderer.register_native_texture(device, &texture_view, wgpu::FilterMode::Linear);

        let (depth_texture, depth_texture_view) = Self::create_depth_texture(device, width, height);

        Self {
            width,
            height,
            texture,
            texture_view,
            texture_id,
            depth_texture,
            depth_texture_view,
        }
    }

    pub(crate) fn resize(&mut self, device: &wgpu::Device, gui: &mut Gui, width: f32, height: f32) {
        let width = (gui.pixels_per_point() * width).round() as u32;
        let height = (gui.pixels_per_point() * height).round() as u32;

        if width == 0 || height == 0 {
            return;
        }

        self.width = width;
        self.height = height;

        let (texture, texture_view) = Self::create_texture(device, self.width, self.height);
        gui.renderer
            .update_egui_texture_from_wgpu_texture(device, &texture_view, wgpu::FilterMode::Linear, self.texture_id);
        self.texture = texture;
        self.texture_view = texture_view;

        let (depth_texture, depth_texture_view) = Self::create_depth_texture(device, self.width, self.height);
        self.depth_texture = depth_texture;
        self.depth_texture_view = depth_texture_view;
    }

    fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("viewport-texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, texture_view)
    }

    fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("viewport-depth-texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (texture, texture_view)
    }
}
