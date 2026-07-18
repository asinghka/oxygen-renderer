use std::num::NonZeroU64;
use wgpu::util::DeviceExt;

pub(crate) struct UniformBinding {
    uniform_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl UniformBinding {
    pub(crate) fn new(device: &wgpu::Device, label: &str, data: &[u8], visibility: wgpu::ShaderStages) -> Self {
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{label}-uniform-buffer")),
            contents: data,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{label}-bind-group-layout")),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(uniform_buffer.size() as u64),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{label}-bind-group")),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            uniform_buffer: uniform_buffer.clone(),
            bind_group_layout,
            bind_group,
        }
    }
    pub(crate) fn uniform_buffer(&self) -> &wgpu::Buffer {
        &self.uniform_buffer
    }

    pub(crate) fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub(crate) fn write(&self, queue: &wgpu::Queue, data: &[u8]) {
        debug_assert_eq!(data.len() as u64, self.uniform_buffer.size());
        queue.write_buffer(&self.uniform_buffer, 0, data);
    }
}
