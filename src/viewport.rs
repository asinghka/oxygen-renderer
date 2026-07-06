use crate::gui::Gui;
use wgpu::{Extent3d, FilterMode, TextureDimension, TextureFormat, TextureUsages};

pub(crate) struct Viewport {
    pub(crate) width: u32,
    pub(crate) height: u32,

    _texture: wgpu::Texture,
    pub(crate) texture_view: wgpu::TextureView,
    pub(crate) texture_id: egui::TextureId,
}

impl Viewport {
    pub(crate) fn new(device: &wgpu::Device, gui: &mut Gui, width: u32, height: u32) -> Self {
        let (texture, texture_view) = Self::create_texture(device, width, height);
        let texture_id = gui.renderer.register_native_texture(device, &texture_view, FilterMode::Linear);

        Self {
            width,
            height,
            _texture: texture,
            texture_view,
            texture_id,
        }
    }

    pub(crate) fn resize(&mut self, device: &wgpu::Device, gui: &mut Gui, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        let (texture, texture_view) = Self::create_texture(device, width, height);
        gui.renderer
            .update_egui_texture_from_wgpu_texture(device, &texture_view, FilterMode::Linear, self.texture_id);

        self._texture = texture;
        self.texture_view = texture_view;
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
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, texture_view)
    }
}
