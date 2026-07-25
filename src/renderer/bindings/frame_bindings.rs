use crate::camera::Camera;
use crate::renderer::RenderSettings;
use crate::scene::Light;
use wgpu::util::DeviceExt;
use wgpu::wgt::SamplerDescriptor;
use wgpu::{TextureDimension, TextureFormat, TextureUsages};

pub(crate) struct FrameBindings {
    camera_uniform_buffer: wgpu::Buffer,
    render_settings_uniform_buffer: wgpu::Buffer,
    light_uniform_buffer: wgpu::Buffer,
    frame_bind_group_layout: wgpu::BindGroupLayout,
    frame_bind_group: wgpu::BindGroup,
    shadow_map_bind_group_layout: wgpu::BindGroupLayout,
    shadow_map_bind_group: wgpu::BindGroup,
    shadow_map_texture_view: wgpu::TextureView,
    shadow_map_sampler: wgpu::Sampler,
    current_shadow_map_resolution: u32,
}

impl FrameBindings {
    pub(crate) fn new(device: &wgpu::Device, light: &Light, settings: &RenderSettings, camera: &Camera) -> Self {
        let render_settings_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("render-settings-uniform-buffer"),
            contents: bytemuck::bytes_of(&settings.uniform()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light-uniform-buffer"),
            contents: bytemuck::bytes_of(&light.uniform()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera-uniform-buffer"),
            contents: bytemuck::bytes_of(&camera.uniform()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let shadow_map_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("shadow-map-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: Some(wgpu::CompareFunction::LessEqual),
            anisotropy_clamp: 1,
            border_color: None,
        });

        let shadow_map_texture_view = create_shadow_map(device, settings.shadow_map_resolution);

        let shadow_map_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow-map-bind-group-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let shadow_map_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow-map-bind-group"),
            layout: &shadow_map_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_uniform_buffer.as_entire_binding(),
            }],
        });

        let frame_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("frame-bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let frame_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("frame-bind-group"),
            layout: &frame_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: render_settings_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: light_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&shadow_map_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&shadow_map_texture_view),
                },
            ],
        });

        Self {
            camera_uniform_buffer,
            render_settings_uniform_buffer,
            light_uniform_buffer,
            frame_bind_group_layout,
            frame_bind_group,
            shadow_map_bind_group_layout,
            shadow_map_bind_group,
            shadow_map_texture_view,
            shadow_map_sampler,
            current_shadow_map_resolution: settings.shadow_map_resolution,
        }
    }

    pub(crate) fn frame_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.frame_bind_group_layout
    }

    pub(crate) fn frame_bind_group(&self) -> &wgpu::BindGroup {
        &self.frame_bind_group
    }

    pub(crate) fn shadow_map_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.shadow_map_bind_group_layout
    }

    pub(crate) fn shadow_map_bind_group(&self) -> &wgpu::BindGroup {
        &self.shadow_map_bind_group
    }

    pub(crate) fn shadow_map_texture_view(&self) -> &wgpu::TextureView {
        &self.shadow_map_texture_view
    }

    pub(crate) fn current_shadow_map_resolution(&self) -> u32 {
        self.current_shadow_map_resolution
    }

    pub(crate) fn update_shadow_map(&mut self, device: &wgpu::Device, resolution: u32) {
        let shadow_map_texture_view = create_shadow_map(device, resolution);

        let frame_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("frame-bind-group"),
            layout: &self.frame_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.camera_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.render_settings_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.light_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.shadow_map_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&shadow_map_texture_view),
                },
            ],
        });

        self.frame_bind_group = frame_bind_group;
        self.shadow_map_texture_view = shadow_map_texture_view;
        self.current_shadow_map_resolution = resolution;
    }

    pub(crate) fn write_all_buffers(&self, queue: &wgpu::Queue, camera: &Camera, settings: &RenderSettings, light: &Light) {
        self.write_camera_buffer(queue, bytemuck::bytes_of(&camera.uniform()));
        self.write_settings_buffer(queue, bytemuck::bytes_of(&settings.uniform()));
        self.write_light_buffer(queue, bytemuck::bytes_of(&light.uniform()));
    }

    fn write_camera_buffer(&self, queue: &wgpu::Queue, data: &[u8]) {
        debug_assert_eq!(data.len() as u64, self.camera_uniform_buffer.size());
        queue.write_buffer(&self.camera_uniform_buffer, 0, data);
    }

    fn write_settings_buffer(&self, queue: &wgpu::Queue, data: &[u8]) {
        debug_assert_eq!(data.len() as u64, self.render_settings_uniform_buffer.size());
        queue.write_buffer(&self.render_settings_uniform_buffer, 0, data);
    }

    fn write_light_buffer(&self, queue: &wgpu::Queue, data: &[u8]) {
        debug_assert_eq!(data.len() as u64, self.light_uniform_buffer.size());
        queue.write_buffer(&self.light_uniform_buffer, 0, data);
    }
}

fn create_shadow_map(device: &wgpu::Device, shadow_map_resolution: u32) -> wgpu::TextureView {
    let shadow_map_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("shadow-map-texture"),
        size: wgpu::Extent3d {
            width: shadow_map_resolution,
            height: shadow_map_resolution,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    shadow_map_texture.create_view(&wgpu::TextureViewDescriptor::default())
}
