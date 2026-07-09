mod gpu;
mod settings;
mod viewport;

pub(crate) use gpu::*;
pub(crate) use settings::*;
pub(crate) use viewport::*;

use crate::camera::Camera;
use crate::mesh;
use crate::mesh::{Scene, Vertex};
use wgpu::util::DeviceExt;
use wgpu::{BindGroupDescriptor, Color, LoadOp, Operations, ShaderSource, StoreOp, TextureFormat};

struct PrimitiveBuffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

pub(crate) struct Renderer {
    render_pipeline: wgpu::RenderPipeline,
    wireframe_pipeline: wgpu::RenderPipeline,

    primitive_buffers: Vec<PrimitiveBuffer>,
    primitive_bind_groups: Vec<wgpu::BindGroup>,

    render_settings_uniform_buffer: wgpu::Buffer,
    render_settings_bind_group: wgpu::BindGroup,

    camera_uniform_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub(crate) fn new(camera: &Camera, gpu: &Gpu, scene: &mut Scene, settings: &RenderSettings) -> Self {
        let shader_module = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
        });

        let loaded_scene = mesh::load("assets/car.glb");
        *scene = loaded_scene;

        let primitive_bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("primitive-bind-group-layout"),
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

        let mut primitive_buffers = Vec::new();
        let mut primitive_bind_groups = Vec::new();
        for primitive in &scene.primitives {
            let vertex_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertex-buffer"),
                contents: bytemuck::cast_slice(&primitive.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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

            let primitive_uniform_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("primitive-buffer"),
                contents: bytemuck::bytes_of(&primitive.uniform()),
                usage: wgpu::BufferUsages::UNIFORM,
            });

            let primitive_bind_group = gpu.device.create_bind_group(&BindGroupDescriptor {
                label: Some("primitive-bind-group"),
                layout: &primitive_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: primitive_uniform_buffer.as_entire_binding(),
                }],
            });

            primitive_bind_groups.push(primitive_bind_group);
        }

        let render_settings_uniform_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("settings-buffer"),
            contents: bytemuck::bytes_of(&settings.uniform()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let render_settings_bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let render_settings_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("render-settings-bind-group"),
            layout: &render_settings_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: render_settings_uniform_buffer.as_entire_binding(),
            }],
        });

        let camera_uniform_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera-buffer"),
            contents: bytemuck::bytes_of(&camera.uniform()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera-bind-group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline-layout"),
            bind_group_layouts: &[
                Some(&camera_bind_group_layout),
                Some(&render_settings_bind_group_layout),
                Some(&primitive_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let render_pipeline = gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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

        let wireframe_pipeline = gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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

        Self {
            render_pipeline,
            wireframe_pipeline,
            primitive_buffers,
            primitive_bind_groups,
            render_settings_uniform_buffer,
            render_settings_bind_group,
            camera_uniform_buffer,
            camera_bind_group,
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

            if settings.wireframe {
                render_pass.set_pipeline(&self.wireframe_pipeline);
            } else {
                render_pass.set_pipeline(&self.render_pipeline);
            }
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
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

    fn update_settings_uniform_buffer(&mut self, settings: &RenderSettings, gpu: &Gpu) {
        gpu.queue
            .write_buffer(&self.render_settings_uniform_buffer, 0, bytemuck::bytes_of(&settings.uniform()));
    }

    fn update_camera_uniform_buffer(&mut self, camera: &Camera, gpu: &Gpu) {
        gpu.queue
            .write_buffer(&self.camera_uniform_buffer, 0, bytemuck::bytes_of(&camera.uniform()));
    }
}
