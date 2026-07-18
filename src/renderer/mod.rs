mod gpu;
mod settings;
pub mod utils;
mod viewport;

pub(crate) use gpu::*;
pub(crate) use settings::*;
pub(crate) use viewport::*;

use crate::camera::Camera;
use crate::renderer::utils::{GridBindings, LightBinding, PrimitiveBindings, UniformBinding};
use crate::scene::{Light, Model, Scene, Vertex};
use wgpu::{Color, LoadOp, Operations, ShaderSource, StoreOp, TextureFormat};

const SHADOW_MAP_SIZE: u32 = 2048;

pub(crate) struct Renderer {
    render_pipeline: wgpu::RenderPipeline,
    wireframe_pipeline: wgpu::RenderPipeline,
    shadow_map_pipeline: wgpu::RenderPipeline,
    line_pipeline: wgpu::RenderPipeline,

    grid_bindings: GridBindings,
    primitive_bindings: PrimitiveBindings,

    light_binding: LightBinding,
    camera_uniform_binding: UniformBinding,
    render_settings_uniform_binding: UniformBinding,
}

impl Renderer {
    pub(crate) fn new(camera: &Camera, light: &Light, gpu: &Gpu, settings: &RenderSettings) -> Self {
        let grid_bindings = GridBindings::new(gpu);
        let primitive_bindings = PrimitiveBindings::new(gpu);

        let render_settings_uniform_binding = UniformBinding::new(
            &gpu.device,
            "render-settings",
            bytemuck::bytes_of(&settings.uniform()),
            wgpu::ShaderStages::FRAGMENT,
        );

        let camera_uniform_binding = UniformBinding::new(
            &gpu.device,
            "camera",
            bytemuck::bytes_of(&camera.uniform()),
            wgpu::ShaderStages::VERTEX_FRAGMENT,
        );

        let light_binding = LightBinding::new(&gpu.device, light);

        let bind_group_layouts = &[
            Some(camera_uniform_binding.bind_group_layout()),
            Some(render_settings_uniform_binding.bind_group_layout()),
            Some(primitive_bindings.bind_group_layout()),
            Some(light_binding.light_bind_group_layout()),
        ];

        let shadow_map_bind_group_layouts = &[
            Some(light_binding.shadow_map_bind_group_layout()),
            Some(primitive_bindings.bind_group_layout()),
        ];

        let grid_bind_group_layouts = &[Some(camera_uniform_binding.bind_group_layout()), Some(grid_bindings.bind_group_layout())];

        let (render_pipeline, wireframe_pipeline, shadow_map_pipeline, line_pipeline) =
            create_pipelines(&gpu.device, bind_group_layouts, grid_bind_group_layouts, shadow_map_bind_group_layouts);

        Self {
            render_pipeline,
            wireframe_pipeline,
            shadow_map_pipeline,
            line_pipeline,
            grid_bindings,
            primitive_bindings,
            light_binding,
            camera_uniform_binding,
            render_settings_uniform_binding,
        }
    }

    pub(crate) fn render(
        &mut self,
        scene: &Scene,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        viewport: &Viewport,
        settings: &mut RenderSettings,
    ) {
        self.render_settings_uniform_binding
            .write(&gpu.queue, bytemuck::bytes_of(&settings.uniform()));
        self.camera_uniform_binding.write(&gpu.queue, bytemuck::bytes_of(&scene.camera.uniform()));
        self.light_binding.write(&gpu.queue, bytemuck::bytes_of(&scene.light.uniform()));

        let invisible = &scene.model.get_invisible_primitives();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow-map-render-pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.light_binding.shadow_map_texture_view(),
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.shadow_map_pipeline);
            render_pass.set_bind_group(0, self.light_binding.shadow_map_bind_group(), &[]);

            for (i, (primitive_buffer, primitive_bind_group)) in self
                .primitive_bindings
                .buffers()
                .iter()
                .zip(self.primitive_bindings.bind_groups().iter())
                .enumerate()
            {
                if invisible.contains(&i) {
                    continue;
                }

                render_pass.set_bind_group(1, primitive_bind_group, &[]);
                render_pass.set_vertex_buffer(0, primitive_buffer.vertex_buffer.slice(..));
                render_pass.set_index_buffer(primitive_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..primitive_buffer.num_indices, 0, 0..1);
            }
        }

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

            render_pass.set_bind_group(0, self.camera_uniform_binding.bind_group(), &[]);

            if settings.grid {
                render_pass.set_pipeline(&self.line_pipeline);

                render_pass.set_bind_group(1, self.grid_bindings.grid_bind_group(), &[]);
                render_pass.set_vertex_buffer(0, self.grid_bindings.grid_buffer().vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.grid_bindings.grid_buffer().index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.grid_bindings.grid_buffer().num_indices, 0, 0..1);

                render_pass.set_bind_group(1, self.grid_bindings.subgrid_bind_group(), &[]);
                render_pass.set_vertex_buffer(0, self.grid_bindings.subgrid_buffer().vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.grid_bindings.subgrid_buffer().index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.grid_bindings.subgrid_buffer().num_indices, 0, 0..1);
            }

            if settings.wireframe {
                render_pass.set_pipeline(&self.wireframe_pipeline);
            } else {
                render_pass.set_pipeline(&self.render_pipeline);
            }

            render_pass.set_bind_group(1, self.render_settings_uniform_binding.bind_group(), &[]);

            for (i, (primitive_buffer, primitive_bind_group)) in self
                .primitive_bindings
                .buffers()
                .iter()
                .zip(self.primitive_bindings.bind_groups().iter())
                .enumerate()
            {
                if invisible.contains(&i) {
                    continue;
                }

                render_pass.set_bind_group(2, primitive_bind_group, &[]);
                render_pass.set_bind_group(3, self.light_binding.light_bind_group(), &[]);

                render_pass.set_vertex_buffer(0, primitive_buffer.vertex_buffer.slice(..));
                render_pass.set_index_buffer(primitive_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..primitive_buffer.num_indices, 0, 0..1);
            }
        }
    }

    pub(crate) fn load(&mut self, gpu: &Gpu, model: &Model) {
        self.primitive_bindings.update_from_model(gpu, model);
    }
}

fn create_pipelines(
    device: &wgpu::Device,
    bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
    grid_bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
    shadow_map_bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
) -> (wgpu::RenderPipeline, wgpu::RenderPipeline, wgpu::RenderPipeline, wgpu::RenderPipeline) {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shader"),
        source: ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("render-pipeline-layout"),
        bind_group_layouts,
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
        label: Some("shadow-map-shader"),
        source: ShaderSource::Wgsl(include_str!("../shaders/shadow.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("shadow-map-pipeline-layout"),
        bind_group_layouts: shadow_map_bind_group_layouts,
        immediate_size: 0,
    });

    let shadow_map_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow-map-pipeline"),
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
            bias: wgpu::DepthBiasState {
                constant: 2,
                slope_scale: 2.0,
                clamp: 0.0,
            },
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: None,
        multiview_mask: None,
        cache: None,
    });

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("grid-shader"),
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

    (render_pipeline, wireframe_pipeline, shadow_map_pipeline, grid_pipeline)
}
