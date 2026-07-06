use crate::camera::{Camera, CameraDescriptor, CameraUniform};
use crate::editor;
use crate::gpu::Gpu;
use crate::gui::Gui;
use crate::input::InputState;
use crate::vertex::{INDICES, VERTICES, Vertex};
use crate::viewport::Viewport;
use wgpu::util::DeviceExt;
use wgpu::{Color, CurrentSurfaceTexture, LoadOp, Operations, ShaderSource, StoreOp, TextureFormat};
use winit::keyboard::KeyCode;
use winit::window::Window;

pub(crate) struct Renderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    viewport: Viewport,
}

impl Renderer {
    pub(crate) fn new(gpu: &Gpu, gui: &mut Gui) -> Self {
        let shader_module = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        let vertex_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex-buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index-buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;

        let camera = Camera::new(&CameraDescriptor {
            eye: glam::vec3(0.0, 0.0, 2.0),
            target: glam::Vec3::ZERO,
            up: glam::Vec3::Y,
            aspect: gpu.config.width as f32 / gpu.config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        });

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_projection_matrix(&camera);

        let camera_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera-buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
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
            bind_group_layouts: &[Some(&camera_bind_group_layout)],
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
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: None,
                compilation_options: Default::default(),
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
            vertex_buffer,
            index_buffer,
            num_indices,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            viewport,
        }
    }

    pub(crate) fn render(&mut self, window: &Window, gpu: &Gpu, gui: &mut Gui) {
        let frame = match gpu.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => frame,
            CurrentSurfaceTexture::Suboptimal(frame) => frame,
            _ => return,
        };

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.viewport.texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        let mut viewport_size = egui::Vec2::ZERO;
        gui.render(window, &gpu.device, &gpu.queue, &mut encoder, &view, |ui| {
            viewport_size = editor::build(ui, self.viewport.texture_id);
        });

        gpu.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        if viewport_size.x > 0.0 && viewport_size.y > 0.0 {
            self.resize_viewport(gpu, gui, viewport_size);
        }
    }

    pub(crate) fn update(&mut self, input_handler: &InputState, gpu: &Gpu) {
        let mut direction = glam::Vec3::ZERO;

        if input_handler.is_pressed(KeyCode::KeyA) {
            direction -= glam::Vec3::X;
        }
        if input_handler.is_pressed(KeyCode::KeyD) {
            direction += glam::Vec3::X;
        }
        if input_handler.is_pressed(KeyCode::KeyQ) {
            direction += glam::Vec3::Y;
        }
        if input_handler.is_pressed(KeyCode::KeyE) {
            direction -= glam::Vec3::Y;
        }
        if input_handler.is_pressed(KeyCode::KeyW) {
            direction -= glam::Vec3::Z;
        }
        if input_handler.is_pressed(KeyCode::KeyS) {
            direction += glam::Vec3::Z;
        }

        if direction != glam::Vec3::ZERO {
            self.camera.update(direction);
            self.update_camera_uniform_buffer(gpu);
        }
    }

    fn resize_viewport(&mut self, gpu: &Gpu, gui: &mut Gui, size: egui::Vec2) {
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

        self.camera.update_aspect_ratio(size.x / size.y);
        self.update_camera_uniform_buffer(gpu);
    }

    fn update_camera_uniform_buffer(&mut self, gpu: &Gpu) {
        self.camera_uniform.update_view_projection_matrix(&self.camera);
        gpu.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }
}
