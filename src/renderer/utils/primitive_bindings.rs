use crate::renderer::Gpu;
use crate::renderer::utils::PrimitiveBuffer;
use crate::scene::Model;
use std::collections::HashSet;
use wgpu::util::DeviceExt;

pub(crate) struct PrimitiveBindings {
    buffers: Vec<PrimitiveBuffer>,

    bind_group_layout: wgpu::BindGroupLayout,
    bind_groups: Vec<wgpu::BindGroup>,
}

impl PrimitiveBindings {
    pub(crate) fn new(gpu: &Gpu) -> Self {
        let bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        Self {
            buffers: Vec::new(),
            bind_group_layout,
            bind_groups: Vec::new(),
        }
    }

    pub(crate) fn update_from_model(&mut self, gpu: &Gpu, model: &Model) {
        let (primitive_buffers, primitive_bind_groups) = build_bindings(&gpu.device, &self.bind_group_layout, model);

        self.buffers = primitive_buffers;
        self.bind_groups = primitive_bind_groups;
    }

    pub(crate) fn visible(&self, invisible: &HashSet<usize>) -> impl Iterator<Item = (&PrimitiveBuffer, &wgpu::BindGroup)> {
        self.buffers
            .iter()
            .zip(self.bind_groups.iter())
            .enumerate()
            .filter(|(i, _)| !invisible.contains(i))
            .map(|(_, (buf, bg))| (buf, bg))
    }

    pub(crate) fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}

fn build_bindings(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout, model: &Model) -> (Vec<PrimitiveBuffer>, Vec<wgpu::BindGroup>) {
    let mut primitive_buffers = Vec::new();
    let mut primitive_bind_groups = Vec::new();

    for (i, primitive) in model.primitives.iter().enumerate() {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("vertex-buffer-{i}")),
            contents: bytemuck::cast_slice(&primitive.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("index-buffer-{i}")),
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
            label: Some(&format!("primitive-uniform-buffer-{i}")),
            contents: bytemuck::bytes_of(&primitive.transform_uniform()),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let primitive_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("primitive-bind-group-{i}")),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: primitive_uniform_buffer.as_entire_binding(),
            }],
        });

        primitive_bind_groups.push(primitive_bind_group);
    }

    (primitive_buffers, primitive_bind_groups)
}
