mod gpu;
mod settings;
mod viewport;

pub(crate) use gpu::*;
pub(crate) use settings::*;
pub(crate) use viewport::*;

use crate::app::FrameStats;
use crate::camera::Camera;
use crate::mesh::{Vertex, model};
use crate::ui::{Gui, editor};
use wgpu::util::DeviceExt;
use wgpu::{Color, CurrentSurfaceTexture, LoadOp, Operations, ShaderSource, StoreOp, TextureFormat};
use winit::window::Window;

pub(crate) struct Renderer {
    render_pipeline: wgpu::RenderPipeline,
    wireframe_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    render_settings_buffer: wgpu::Buffer,
    render_settings_bind_group: wgpu::BindGroup,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    viewport: Viewport,
}

impl Renderer {
    pub(crate) fn new(camera: &Camera, gpu: &Gpu, gui: &mut Gui, settings: &RenderSettings, stats: &mut FrameStats) -> Self {
        let shader_module = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
        });

        let (vertices, indices) = model::load("assets/dragon.glb");

        stats.set_model(vertices.len() as u32, indices.len() as u32);

        let vertex_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex-buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index-buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = indices.len() as u32;

        let render_settings_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
                resource: render_settings_buffer.as_entire_binding(),
            }],
        });

        let camera_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera-buffer"),
            contents: bytemuck::bytes_of(&camera.uniform()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera-bind-group-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
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
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline-layout"),
            bind_group_layouts: &[Some(&camera_bind_group_layout), Some(&render_settings_bind_group_layout)],
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
                    format: TextureFormat::Rgba8Unorm,
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
                    format: TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: Default::default(),
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let viewport = Viewport::new(&gpu.device, gui, gpu.config.width, gpu.config.height);

        Self {
            render_pipeline,
            wireframe_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            render_settings_buffer,
            render_settings_bind_group,
            camera_buffer,
            camera_bind_group,
            viewport,
        }
    }

    pub(crate) fn render(
        &mut self,
        window: &Window,
        camera: &mut Camera,
        gpu: &Gpu,
        gui: &mut Gui,
        settings: &mut RenderSettings,
        stats: &FrameStats,
    ) -> egui::Rect {
        let frame = match gpu.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => frame,
            CurrentSurfaceTexture::Suboptimal(frame) => frame,
            _ => return egui::Rect::NOTHING,
        };

        self.update_settings_uniform_buffer(settings, gpu);
        self.update_camera_uniform_buffer(camera, gpu);

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let mut viewport_rect = egui::Rect::NOTHING;
        gui.render(window, &gpu.device, &gpu.queue, &mut encoder, &view, |ui| {
            viewport_rect = editor::build(ui, self.viewport.texture_id, settings, stats);
        });

        if viewport_rect.size().x > 0.0 && viewport_rect.size().y > 0.0 {
            self.resize_viewport(camera, gpu, gui, viewport_rect.size());
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.viewport.texture_view,
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
                    view: &self.viewport.depth_texture_view,
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

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        viewport_rect
    }

    fn resize_viewport(&mut self, camera: &mut Camera, gpu: &Gpu, gui: &mut Gui, size: egui::Vec2) {
        let pixels_per_point = gui.pixels_per_point();
        let width = (size.x * pixels_per_point).round() as u32;
        let height = (size.y * pixels_per_point).round() as u32;

        if width == 0 || height == 0 {
            return;
        }

        if self.viewport.width == width && self.viewport.height == height {
            return;
        }

        self.viewport.resize(&gpu.device, gui, width, height);
        camera.update_aspect_ratio(size.x / size.y);
    }

    fn update_settings_uniform_buffer(&mut self, settings: &RenderSettings, gpu: &Gpu) {
        gpu.queue
            .write_buffer(&self.render_settings_buffer, 0, bytemuck::bytes_of(&settings.uniform()));
    }

    fn update_camera_uniform_buffer(&mut self, camera: &Camera, gpu: &Gpu) {
        gpu.queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&camera.uniform()));
    }
}
