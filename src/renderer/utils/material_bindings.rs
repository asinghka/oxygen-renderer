use crate::renderer::Gpu;
use crate::scene::Model;
use wgpu::util::{BufferInitDescriptor, DeviceExt, TextureDataOrder};
use wgpu::wgt::SamplerDescriptor;
use wgpu::{TexelCopyBufferLayout, TextureDimension, TextureFormat, TextureUsages};

pub(crate) struct MaterialBindings {
    material_uniform_buffers: Vec<wgpu::Buffer>,
    bind_group_layout: wgpu::BindGroupLayout,
    material_bind_groups: Vec<wgpu::BindGroup>,

    texture_views: Vec<Option<wgpu::TextureView>>,
    texture_sampler: wgpu::Sampler,

    placeholder_texture_view: wgpu::TextureView,
}

impl MaterialBindings {
    pub(crate) fn new(gpu: &Gpu) -> Self {
        let bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material-bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
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
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let texture_sampler = gpu.device.create_sampler(&SamplerDescriptor {
            label: Some("texture-sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let placeholder_texture_view = create_placeholder_texture(gpu);

        Self {
            material_uniform_buffers: Vec::new(),
            bind_group_layout,
            material_bind_groups: Vec::new(),
            texture_views: Vec::new(),
            texture_sampler,
            placeholder_texture_view,
        }
    }

    pub(crate) fn update_from_model(&mut self, gpu: &Gpu, model: &Model) {
        self.texture_views = create_texture_views(gpu, model);

        let (material_uniform_buffers, material_bind_groups) = build_bindings(
            &gpu.device,
            &self.bind_group_layout,
            model,
            &self.texture_views,
            &self.texture_sampler,
            &self.placeholder_texture_view,
        );

        self.material_uniform_buffers = material_uniform_buffers;
        self.material_bind_groups = material_bind_groups;
    }

    pub(crate) fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}

fn build_bindings(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    model: &Model,
    texture_views: &Vec<Option<wgpu::TextureView>>,
    texture_sampler: &wgpu::Sampler,
    placeholder_texture_view: &wgpu::TextureView,
) -> (Vec<wgpu::Buffer>, Vec<wgpu::BindGroup>) {
    let mut material_buffers = Vec::new();
    let mut material_bind_groups = Vec::new();

    for (i, material) in model.materials.iter().enumerate() {
        let material_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("material-buffer-{}", i)),
            contents: bytemuck::bytes_of(&material.uniform()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let albedo_texture_view = if let Some(index) = material.albedo_texture {
            texture_views[index].as_ref().unwrap_or(placeholder_texture_view)
        } else {
            placeholder_texture_view
        };

        let normal_texture_view = if let Some(index) = material.normal_texture {
            texture_views[index].as_ref().unwrap_or(placeholder_texture_view)
        } else {
            placeholder_texture_view
        };

        material_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("material-bind-group-{}", i)),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: material_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&albedo_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
        }));

        material_buffers.push(material_uniform_buffer);
    }

    (material_buffers, material_bind_groups)
}

fn create_texture_views(gpu: &Gpu, model: &Model) -> Vec<Option<wgpu::TextureView>> {
    model
        .textures
        .iter()
        .enumerate()
        .map(|(i, tex_data)| {
            let tex_data = tex_data.as_ref()?;

            let size = wgpu::Extent3d {
                width: tex_data.width,
                height: tex_data.height,
                depth_or_array_layers: 1,
            };

            let format = if tex_data.srgb {
                TextureFormat::Rgba8UnormSrgb
            } else {
                TextureFormat::Rgba8Unorm
            };

            let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("scene-texture-{i}")),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            });

            gpu.queue.write_texture(
                texture.as_image_copy(),
                &tex_data.pixels,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * size.width),
                    rows_per_image: Some(size.height),
                },
                size,
            );

            Some(texture.create_view(&wgpu::TextureViewDescriptor::default()))
        })
        .collect()
}

fn create_placeholder_texture(gpu: &Gpu) -> wgpu::TextureView {
    let placeholder_texture = &gpu.device.create_texture_with_data(
        &gpu.queue,
        &wgpu::TextureDescriptor {
            label: Some("placeholder-texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        TextureDataOrder::LayerMajor,
        &[255_u8; 4],
    );

    placeholder_texture.create_view(&wgpu::TextureViewDescriptor::default())
}
