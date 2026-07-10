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
    pub(crate) fn grid(size: f32, divisions: u32) -> Self {
        let half = size / 2.0;
        let step = size / divisions as f32;
        let normal = [0.0, 1.0, 0.0];

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for n in 0..=divisions {
            let offset = -half + n as f32 * step;

            let base = vertices.len() as u32;
            vertices.push(Vertex {
                position: [-half, 0.0, offset],
                normal,
            });
            vertices.push(Vertex {
                position: [half, 0.0, offset],
                normal,
            });
            indices.push(base);
            indices.push(base + 1);

            let base = vertices.len() as u32;
            vertices.push(Vertex {
                position: [offset, 0.0, -half],
                normal,
            });
            vertices.push(Vertex {
                position: [offset, 0.0, half],
                normal,
            });
            indices.push(base);
            indices.push(base + 1);
        }

        Self {
            vertices,
            indices,
            model: glam::Mat4::from_translation(-glam::Vec3::Y),
            color: [0.5, 0.5, 0.5, 1.0],
        }
    }

    pub(crate) fn uniform(&self) -> PrimitiveUniform {
        PrimitiveUniform {
            model: self.model.to_cols_array_2d(),
            normal_model: self.model.inverse().transpose().to_cols_array_2d(),
            color: self.color,
        }
    }
}
