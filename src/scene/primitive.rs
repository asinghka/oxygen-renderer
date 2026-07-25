use crate::scene::Vertex;
use std::mem::offset_of;

const _: () = assert!(offset_of!(TransformUniform, normal_model) == 64);
const _: () = assert!(offset_of!(TransformUniform, scale) == 128);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct TransformUniform {
    model: [[f32; 4]; 4],
    normal_model: [[f32; 4]; 4],
    pub(crate) scale: f32,
    _pad: [f32; 3],
}

pub(crate) struct Primitive {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
    pub(crate) model: glam::Mat4,
    pub(crate) material: Option<usize>,
}

impl Primitive {
    pub(crate) fn transform_uniform(&self) -> TransformUniform {
        TransformUniform {
            model: self.model.to_cols_array_2d(),
            normal_model: self.model.inverse().transpose().to_cols_array_2d(),
            scale: 1.0,
            _pad: [0.0; 3],
        }
    }
}
