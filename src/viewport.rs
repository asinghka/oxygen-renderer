use crate::gui::Gui;
use wgpu::{Extent3d, FilterMode, TextureDimension, TextureFormat, TextureUsages};

pub(crate) struct Viewport {
    _texture: wgpu::Texture,
    pub(crate) texture_view: wgpu::TextureView,
    pub(crate) texture_id: egui::TextureId,
}

impl Viewport {
    pub(crate) fn new(device: &wgpu::Device, gui: &mut Gui, width: u32, height: u32) -> Self {
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

        let texture_id = gui.renderer.register_native_texture(device, &texture_view, FilterMode::Linear);

        Self {
            _texture: texture,
            texture_view,
            texture_id,
        }
    }
}
