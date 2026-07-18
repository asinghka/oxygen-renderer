use crate::renderer::Gpu;
use crate::renderer::utils::PrimitiveBuffer;
use crate::scene::Model;
use wgpu::util::{DeviceExt, TextureDataOrder};
use wgpu::wgt::SamplerDescriptor;
use wgpu::{TexelCopyBufferLayout, TextureDimension, TextureFormat, TextureUsages};

pub(crate) struct PrimitiveBindings {
    buffers: Vec<PrimitiveBuffer>,

    bind_groups: Vec<wgpu::BindGroup>,
    bind_group_layout: wgpu::BindGroupLayout,

    texture_sampler: wgpu::Sampler,
    texture_views: Vec<Option<wgpu::TextureView>>,

    placeholder_view: wgpu::TextureView,
}

impl PrimitiveBindings {
    pub(crate) fn new(gpu: &Gpu) -> Self {
        let bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("primitive-bind-group-layout"),
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
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
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
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
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

        let placeholder_view = create_placeholder_texture(gpu);

        Self {
            buffers: Vec::new(),
            bind_groups: Vec::new(),
            bind_group_layout,
            texture_sampler,
            texture_views: Vec::new(),
            placeholder_view,
        }
    }

    pub(crate) fn update_from_model(&mut self, gpu: &Gpu, model: &Model) {
        self.texture_views = create_texture_views(gpu, model);

        let (primitive_buffers, primitive_bind_groups) = build_bindings(
            &gpu.device,
            &self.bind_group_layout,
            &self.texture_views,
            &self.texture_sampler,
            &self.placeholder_view,
            model,
        );

        self.buffers = primitive_buffers;
        self.bind_groups = primitive_bind_groups;
    }

    pub(crate) fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub(crate) fn buffers(&self) -> &Vec<PrimitiveBuffer> {
        &self.buffers
    }

    pub(crate) fn bind_groups(&self) -> &Vec<wgpu::BindGroup> {
        &self.bind_groups
    }
}

fn build_bindings(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    texture_views: &[Option<wgpu::TextureView>],
    texture_sampler: &wgpu::Sampler,
    placeholder_view: &wgpu::TextureView,
    model: &Model,
) -> (Vec<PrimitiveBuffer>, Vec<wgpu::BindGroup>) {
    let mut primitive_buffers = Vec::new();
    let mut primitive_bind_groups = Vec::new();

    for primitive in &model.primitives {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex-buffer"),
            contents: bytemuck::cast_slice(&primitive.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index-buffer"),
            contents: bytemuck::cast_slice(&primitive.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = primitive.indices.len() as u32;

        primitive_buffers.push(PrimitiveBuffer {
            vertex_buffer,
            index_buffer,
            num_indices,
        });

        let primitive_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("primitive-buffer"),
            contents: bytemuck::bytes_of(&primitive.uniform()),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let albedo_texture_view = primitive
            .albedo_texture
            .and_then(|index| texture_views[index].as_ref())
            .unwrap_or(placeholder_view);

        let normal_texture_view = primitive
            .normal_texture
            .and_then(|index| texture_views[index].as_ref())
            .unwrap_or(placeholder_view);

        let primitive_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("primitive-bind-group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: primitive_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(texture_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(albedo_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(normal_texture_view),
                },
            ],
        });

        primitive_bind_groups.push(primitive_bind_group);
    }

    (primitive_buffers, primitive_bind_groups)
}

fn create_texture_views(gpu: &Gpu, model: &Model) -> Vec<Option<wgpu::TextureView>> {
    model
        .textures
        .iter()
        .map(|tex_data| {
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
                label: Some("scene-texture"),
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
