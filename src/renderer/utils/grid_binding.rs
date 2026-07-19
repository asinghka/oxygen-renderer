use crate::renderer::Gpu;
use crate::renderer::utils::PrimitiveBuffer;
use crate::scene::Primitive;
use wgpu::util::DeviceExt;

pub(crate) struct GridBindings {
    grid_buffer: PrimitiveBuffer,
    grid_bind_group: wgpu::BindGroup,

    subgrid_buffer: PrimitiveBuffer,
    subgrid_bind_group: wgpu::BindGroup,

    bind_group_layout: wgpu::BindGroupLayout,
}

impl GridBindings {
    pub(crate) fn new(gpu: &Gpu) -> Self {
        let bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let (grid_buffer, grid_bind_group) = build_grid_binding(&gpu.device, &bind_group_layout);
        let (subgrid_buffer, subgrid_bind_group) = build_subgrid_binding(&gpu.device, &bind_group_layout);

        Self {
            grid_buffer,
            grid_bind_group,
            subgrid_buffer,
            subgrid_bind_group,
            bind_group_layout,
        }
    }

    pub(crate) fn grid_buffer(&self) -> &PrimitiveBuffer {
        &self.grid_buffer
    }

    pub(crate) fn grid_bind_group(&self) -> &wgpu::BindGroup {
        &self.grid_bind_group
    }

    pub(crate) fn subgrid_buffer(&self) -> &PrimitiveBuffer {
        &self.subgrid_buffer
    }

    pub(crate) fn subgrid_bind_group(&self) -> &wgpu::BindGroup {
        &self.subgrid_bind_group
    }

    pub(crate) fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub(crate) fn record_grid(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(1, self.grid_bind_group(), &[]);
        self.grid_buffer.record(render_pass);
    }

    pub(crate) fn record_subgrid(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(1, self.subgrid_bind_group(), &[]);
        self.subgrid_buffer.record(render_pass);
    }
}

fn build_grid_binding(device: &wgpu::Device, grid_bind_group_layout: &wgpu::BindGroupLayout) -> (PrimitiveBuffer, wgpu::BindGroup) {
    let grid_primitive = Primitive::grid(30.0, 16);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("grid-vertex-buffer"),
        contents: bytemuck::cast_slice(&grid_primitive.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("grid-index-buffer"),
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
        label: Some("grid-primitive-buffer"),
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

fn build_subgrid_binding(device: &wgpu::Device, grid_bind_group_layout: &wgpu::BindGroupLayout) -> (PrimitiveBuffer, wgpu::BindGroup) {
    let grid_primitive = Primitive::subgrid(30.0, 16);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("subgrid-vertex-buffer"),
        contents: bytemuck::cast_slice(&grid_primitive.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("subgrid-index-buffer"),
        contents: bytemuck::cast_slice(&grid_primitive.indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let num_indices = grid_primitive.indices.len() as u32;

    let subgrid_buffer = PrimitiveBuffer {
        vertex_buffer,
        index_buffer,
        num_indices,
    };

    let primitive_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("subgrid-primitive-buffer"),
        contents: bytemuck::bytes_of(&grid_primitive.uniform()),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let subgrid_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("subgrid-bind-group"),
        layout: grid_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: primitive_uniform_buffer.as_entire_binding(),
        }],
    });

    (subgrid_buffer, subgrid_bind_group)
}
