use crate::renderer::SHADOW_MAP_SIZE;
use crate::scene::Light;
use wgpu::util::DeviceExt;
use wgpu::wgt::SamplerDescriptor;
use wgpu::{TextureDimension, TextureFormat, TextureUsages};

pub(crate) struct LightBinding {
    light_uniform_buffer: wgpu::Buffer,
    light_bind_group_layout: wgpu::BindGroupLayout,
    light_bind_group: wgpu::BindGroup,
    shadow_map_bind_group_layout: wgpu::BindGroupLayout,
    shadow_map_bind_group: wgpu::BindGroup,
    shadow_map_texture_view: wgpu::TextureView,
}

impl LightBinding {
    pub(crate) fn new(device: &wgpu::Device, light: &Light) -> Self {
        let (shadow_map_texture_view, shadow_map_sampler) = create_shadow_map(device);

        let light_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light-uniform-buffer"),
            contents: bytemuck::bytes_of(&light.uniform()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

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

        let light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("light-bind-group-layout"),
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
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
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

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light-bind-group"),
            layout: &light_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shadow_map_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&shadow_map_texture_view),
                },
            ],
        });

        Self {
            light_uniform_buffer,
            light_bind_group_layout,
            light_bind_group,
            shadow_map_bind_group_layout,
            shadow_map_bind_group,
            shadow_map_texture_view,
        }
    }

    pub(crate) fn light_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.light_bind_group_layout
    }

    pub(crate) fn light_bind_group(&self) -> &wgpu::BindGroup {
        &self.light_bind_group
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

    pub(crate) fn write(&self, queue: &wgpu::Queue, data: &[u8]) {
        debug_assert_eq!(data.len() as u64, self.light_uniform_buffer.size());
        queue.write_buffer(&self.light_uniform_buffer, 0, data);
    }
}

fn create_shadow_map(device: &wgpu::Device) -> (wgpu::TextureView, wgpu::Sampler) {
    let shadow_map_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("shadow-map-texture"),
        size: wgpu::Extent3d {
            width: SHADOW_MAP_SIZE,
            height: SHADOW_MAP_SIZE,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let shadow_map_sampler = device.create_sampler(&SamplerDescriptor {
        label: Some("shadow-map-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        lod_min_clamp: 0.0,
        lod_max_clamp: 0.0,
        compare: Some(wgpu::CompareFunction::LessEqual),
        anisotropy_clamp: 1,
        border_color: None,
    });

    let shadow_map_texture_view = shadow_map_texture.create_view(&wgpu::TextureViewDescriptor::default());

    (shadow_map_texture_view, shadow_map_sampler)
}
