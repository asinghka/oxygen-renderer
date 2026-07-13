mod gpu;
mod settings;
mod viewport;

pub(crate) use gpu::*;
pub(crate) use settings::*;
pub(crate) use viewport::*;

use crate::camera::Camera;
use crate::mesh::{Primitive, Scene, Vertex};
use wgpu::util::DeviceExt;
use wgpu::wgt::{SamplerDescriptor, TextureDataOrder};
use wgpu::{Color, LoadOp, Operations, ShaderSource, StoreOp, TexelCopyBufferLayout, TextureDimension, TextureFormat, TextureUsages};

struct PrimitiveBuffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

pub(crate) struct Renderer {
    render_pipeline: wgpu::RenderPipeline,
    wireframe_pipeline: wgpu::RenderPipeline,
    line_pipeline: wgpu::RenderPipeline,

    grid_buffer: PrimitiveBuffer,
    grid_bind_group: wgpu::BindGroup,

    primitive_buffers: Vec<PrimitiveBuffer>,
    primitive_bind_groups: Vec<wgpu::BindGroup>,
    primitive_bind_group_layout: wgpu::BindGroupLayout,

    render_settings_uniform_buffer: wgpu::Buffer,
    render_settings_bind_group: wgpu::BindGroup,

    camera_uniform_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    texture_sampler: wgpu::Sampler,
    texture_views: Vec<Option<wgpu::TextureView>>,
    placeholder_view: wgpu::TextureView,
}

impl Renderer {
    pub(crate) fn new(camera: &Camera, gpu: &Gpu, settings: &RenderSettings) -> Self {
        let primitive_bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let grid_bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("grid-bind-group-layout"),
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

        let placeholder_texture = gpu.device.create_texture_with_data(
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

        let placeholder_view = placeholder_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let (grid_buffer, grid_bind_group) = build_grid(&gpu.device, &grid_bind_group_layout);
        let (render_settings_uniform_buffer, render_settings_bind_group_layout, render_settings_bind_group) =
            build_render_settings(&gpu.device, settings);
        let (camera_uniform_buffer, camera_bind_group_layout, camera_bind_group) = build_camera(&gpu.device, camera);

        let primitive_bind_group_layouts = &[
            Some(&camera_bind_group_layout),
            Some(&render_settings_bind_group_layout),
            Some(&primitive_bind_group_layout),
        ];

        let grid_bind_group_layouts = &[Some(&camera_bind_group_layout), Some(&grid_bind_group_layout)];

        let (render_pipeline, wireframe_pipeline, line_pipeline) =
            build_pipelines(&gpu.device, primitive_bind_group_layouts, grid_bind_group_layouts);

        Self {
            render_pipeline,
            wireframe_pipeline,
            line_pipeline,
            grid_buffer,
            grid_bind_group,
            primitive_buffers: Vec::new(),
            primitive_bind_groups: Vec::new(),
            primitive_bind_group_layout,
            render_settings_uniform_buffer,
            render_settings_bind_group,
            camera_uniform_buffer,
            camera_bind_group,
            texture_sampler,
            texture_views: Vec::new(),
            placeholder_view,
        }
    }

    pub(crate) fn render(
        &mut self,
        camera: &mut Camera,
        scene: &Scene,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        viewport: &Viewport,
        settings: &mut RenderSettings,
    ) {
        self.update_settings_uniform_buffer(settings, gpu);
        self.update_camera_uniform_buffer(camera, gpu);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &viewport.texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: settings.background[0] as f64,
                            g: settings.background[1] as f64,
                            b: settings.background[2] as f64,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &viewport.depth_texture_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            if settings.grid {
                render_pass.set_pipeline(&self.line_pipeline);
                render_pass.set_bind_group(1, &self.grid_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.grid_buffer.vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.grid_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.grid_buffer.num_indices, 0, 0..1);
            }

            if settings.wireframe {
                render_pass.set_pipeline(&self.wireframe_pipeline);
            } else {
                render_pass.set_pipeline(&self.render_pipeline);
            }

            render_pass.set_bind_group(1, &self.render_settings_bind_group, &[]);

            let invisible = scene.get_invisible_primitives();

            for (i, (primitive_buffer, primitive_bind_group)) in self.primitive_buffers.iter().zip(self.primitive_bind_groups.iter()).enumerate() {
                if invisible.contains(&i) {
                    continue;
                }

                render_pass.set_bind_group(2, primitive_bind_group, &[]);
                render_pass.set_vertex_buffer(0, primitive_buffer.vertex_buffer.slice(..));
                render_pass.set_index_buffer(primitive_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..primitive_buffer.num_indices, 0, 0..1);
            }
        }
    }

    pub(crate) fn load(&mut self, gpu: &Gpu, scene: &Scene) {
        self.texture_views = create_texture_views(&gpu.device, &gpu.queue, scene);

        let (primitive_buffers, primitive_bind_groups) = build_primitives(
            &gpu.device,
            &self.primitive_bind_group_layout,
            &self.texture_views,
            &self.texture_sampler,
            &self.placeholder_view,
            scene,
        );
        self.primitive_buffers = primitive_buffers;
        self.primitive_bind_groups = primitive_bind_groups;
    }

    fn update_settings_uniform_buffer(&mut self, settings: &RenderSettings, gpu: &Gpu) {
        gpu.queue
            .write_buffer(&self.render_settings_uniform_buffer, 0, bytemuck::bytes_of(&settings.uniform()));
    }

    fn update_camera_uniform_buffer(&mut self, camera: &Camera, gpu: &Gpu) {
        gpu.queue
            .write_buffer(&self.camera_uniform_buffer, 0, bytemuck::bytes_of(&camera.uniform()));
    }
}

fn build_pipelines(
    device: &wgpu::Device,
    primitive_bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
    grid_bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
) -> (wgpu::RenderPipeline, wgpu::RenderPipeline, wgpu::RenderPipeline) {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shader"),
        source: ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("render-pipeline-layout"),
        bind_group_layouts: primitive_bind_group_layouts,
        immediate_size: 0,
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("render-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: None,
            compilation_options: Default::default(),
            buffers: &[Vertex::layout()],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: TextureFormat::Rgba8UnormSrgb,
                blend: None,
                write_mask: Default::default(),
            })],
        }),
        multiview_mask: None,
        cache: None,
    });

    let wireframe_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("wireframe-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: None,
            compilation_options: Default::default(),
            buffers: &[Vertex::layout()],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Line,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: TextureFormat::Rgba8UnormSrgb,
                blend: None,
                write_mask: Default::default(),
            })],
        }),
        multiview_mask: None,
        cache: None,
    });

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shader"),
        source: ShaderSource::Wgsl(include_str!("../shaders/grid.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("grid-pipeline-layout"),
        bind_group_layouts: grid_bind_group_layouts,
        immediate_size: 0,
    });

    let grid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("grid-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: None,
            compilation_options: Default::default(),
            buffers: &[Vertex::layout()],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Line,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: TextureFormat::Rgba8UnormSrgb,
                blend: None,
                write_mask: Default::default(),
            })],
        }),
        multiview_mask: None,
        cache: None,
    });

    (render_pipeline, wireframe_pipeline, grid_pipeline)
}

fn build_grid(device: &wgpu::Device, grid_bind_group_layout: &wgpu::BindGroupLayout) -> (PrimitiveBuffer, wgpu::BindGroup) {
    let grid_primitive = Primitive::grid(30.0, 16);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("vertex-buffer"),
        contents: bytemuck::cast_slice(&grid_primitive.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("index-buffer"),
        contents: bytemuck::cast_slice(&grid_primitive.indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let num_indices = grid_primitive.indices.len() as u32;

    let grid_buffer = PrimitiveBuffer {
        vertex_buffer,
        index_buffer,
        num_indices,
    };

    let primitive_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("primitive-buffer"),
        contents: bytemuck::bytes_of(&grid_primitive.uniform()),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let grid_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("grid-bind-group"),
        layout: grid_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: primitive_uniform_buffer.as_entire_binding(),
        }],
    });

    (grid_buffer, grid_bind_group)
}

fn build_primitives(
    device: &wgpu::Device,
    primitive_bind_group_layout: &wgpu::BindGroupLayout,
    texture_views: &[Option<wgpu::TextureView>],
    texture_sampler: &wgpu::Sampler,
    placeholder_view: &wgpu::TextureView,
    scene: &Scene,
) -> (Vec<PrimitiveBuffer>, Vec<wgpu::BindGroup>) {
    let mut primitive_buffers = Vec::new();
    let mut primitive_bind_groups = Vec::new();

    for primitive in &scene.primitives {
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
            layout: primitive_bind_group_layout,
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

fn create_texture_views(device: &wgpu::Device, queue: &wgpu::Queue, scene: &Scene) -> Vec<Option<wgpu::TextureView>> {
    scene
        .textures
        .iter()
        .map(|tex_data| {
            let tex_data = tex_data.as_ref()?;

            let size = wgpu::Extent3d {
                width: tex_data.width,
                height: tex_data.height,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("base-color"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            });

            queue.write_texture(
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

fn build_render_settings(device: &wgpu::Device, settings: &RenderSettings) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
    let render_settings_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("settings-buffer"),
        contents: bytemuck::bytes_of(&settings.uniform()),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let render_settings_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("render-settings-bind-group-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let render_settings_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("render-settings-bind-group"),
        layout: &render_settings_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: render_settings_uniform_buffer.as_entire_binding(),
        }],
    });

    (
        render_settings_uniform_buffer,
        render_settings_bind_group_layout,
        render_settings_bind_group,
    )
}

fn build_camera(device: &wgpu::Device, camera: &Camera) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
    let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("camera-buffer"),
        contents: bytemuck::bytes_of(&camera.uniform()),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("camera-bind-group-layout"),
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

    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("camera-bind-group"),
        layout: &camera_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_uniform_buffer.as_entire_binding(),
        }],
    });

    (camera_uniform_buffer, camera_bind_group_layout, camera_bind_group)
}
