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

impl PrimitiveBuffer {
    pub(crate) fn record(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}
