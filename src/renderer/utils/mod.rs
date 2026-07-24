mod frame_bindings;
mod grid_binding;
mod material_bindings;
mod primitive_bindings;

pub(crate) use frame_bindings::*;
pub(crate) use grid_binding::*;
pub(crate) use material_bindings::*;
pub(crate) use primitive_bindings::*;

pub(crate) struct PrimitiveBuffer {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) num_indices: u32,
}
