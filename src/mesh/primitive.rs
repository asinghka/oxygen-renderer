use crate::mesh::Vertex;
use std::mem::offset_of;

const _: () = assert!(size_of::<PrimitiveUniform>() == 144);
const _: () = assert!(offset_of!(PrimitiveUniform, normal_model) == 64);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct PrimitiveUniform {
    model: [[f32; 4]; 4],
    normal_model: [[f32; 4]; 4],
    color: [f32; 4],
}

pub(crate) struct Primitive {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
    pub(crate) model: glam::Mat4,
    pub(crate) color: [f32; 4],
}

impl Primitive {
    pub(crate) fn uniform(&self) -> PrimitiveUniform {
        PrimitiveUniform {
            model: self.model.to_cols_array_2d(),
            normal_model: self.model.inverse().transpose().to_cols_array_2d(),
            color: self.color,
        }
    }
}
