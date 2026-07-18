mod grid_binding;
mod light_binding;
mod primitive_binding;
mod uniform_binding;

pub(crate) use grid_binding::*;
pub(crate) use light_binding::*;
pub(crate) use primitive_binding::*;
pub(crate) use uniform_binding::*;

pub(crate) struct PrimitiveBuffer {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) num_indices: u32,
}
