use crate::renderer::Gpu;
use crate::renderer::bindings::PrimitiveBuffer;
use crate::scene::Model;
use std::collections::HashSet;
use wgpu::util::DeviceExt;

pub(crate) struct PrimitiveBinding {
    pub(crate) primitive_buffer: PrimitiveBuffer,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) material: Option<usize>,
}

pub(crate) struct PrimitiveBindings {
    primitive_bindings: Vec<PrimitiveBinding>,
    bind_group_layout: wgpu::BindGroupLayout,
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
            primitive_bindings: Vec::new(),
            bind_group_layout,
        }
    }

    pub(crate) fn update_from_model(&mut self, gpu: &Gpu, model: &Model) {
        let primitive_bindings = build_bindings(&gpu.device, &self.bind_group_layout, model);

        self.primitive_bindings = primitive_bindings
    }

    pub(crate) fn visible(&self, invisible: &HashSet<usize>) -> impl Iterator<Item = &PrimitiveBinding> {
        self.primitive_bindings
            .iter()
            .enumerate()
            .filter(|(i, _)| !invisible.contains(i))
            .map(|(_, b)| b)
    }

    pub(crate) fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}

fn build_bindings(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout, model: &Model) -> Vec<PrimitiveBinding> {
    let mut primitive_bindings = Vec::new();

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

        let primitive_buffer = PrimitiveBuffer {
            vertex_buffer,
            index_buffer,
            num_indices,
        };

        let primitive_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("primitive-uniform-buffer-{i}")),
            contents: bytemuck::bytes_of(&primitive.transform_uniform()),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("primitive-bind-group-{i}")),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: primitive_uniform_buffer.as_entire_binding(),
            }],
        });

        primitive_bindings.push(PrimitiveBinding {
            primitive_buffer,
            bind_group,
            material: primitive.material,
        });
    }

    primitive_bindings
}
