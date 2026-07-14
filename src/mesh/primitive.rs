use crate::mesh::Vertex;
use std::mem::offset_of;

const _: () = assert!(size_of::<PrimitiveUniform>() == 144);
const _: () = assert!(offset_of!(PrimitiveUniform, normal_model) == 64);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct PrimitiveUniform {
    model: [[f32; 4]; 4],
    normal_model: [[f32; 4]; 4],
    color: [f32; 3],
    bump: f32,
}

pub(crate) struct Primitive {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
    pub(crate) model: glam::Mat4,
    pub(crate) color: [f32; 3],
    pub(crate) albedo_texture: Option<usize>,
    pub(crate) normal_texture: Option<usize>,
    pub(crate) bump: f32,
}

impl Primitive {
    pub(crate) fn grid(size: f32, divisions: u32) -> Self {
        let step = size / divisions as f32;
        let offsets = (0..=divisions).map(|n| -size / 2.0 + n as f32 * step);

        Self::grid_lines(size, offsets, [0.5, 0.5, 0.5])
    }

    pub(crate) fn subgrid(size: f32, divisions: u32) -> Self {
        let step = size / divisions as f32;
        let offsets = (0..divisions).map(|n| -size / 2.0 + (n as f32 + 0.5) * step);

        Self::grid_lines(size, offsets, [0.2, 0.2, 0.2])
    }

    fn grid_lines(size: f32, offsets: impl Iterator<Item = f32>, color: [f32; 3]) -> Self {
        let half = size / 2.0;
        let normal = [0.0, 1.0, 0.0];

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let uv = [0.0; 2];
        let tangent = [1.0, 0.0, 0.0, 1.0];

        for offset in offsets {
            let base = vertices.len() as u32;
            vertices.push(Vertex {
                position: [-half, 0.0, offset],
                normal,
                uv,
                tangent,
            });
            vertices.push(Vertex {
                position: [half, 0.0, offset],
                normal,
                uv,
                tangent,
            });
            indices.push(base);
            indices.push(base + 1);

            let base = vertices.len() as u32;
            vertices.push(Vertex {
                position: [offset, 0.0, -half],
                normal,
                uv,
                tangent,
            });
            vertices.push(Vertex {
                position: [offset, 0.0, half],
                normal,
                uv,
                tangent,
            });
            indices.push(base);
            indices.push(base + 1);
        }

        Self {
            vertices,
            indices,
            model: glam::Mat4::from_translation(-glam::Vec3::Y),
            color,
            albedo_texture: None,
            normal_texture: None,
            bump: 0.0,
        }
    }

    pub(crate) fn uniform(&self) -> PrimitiveUniform {
        PrimitiveUniform {
            model: self.model.to_cols_array_2d(),
            normal_model: self.model.inverse().transpose().to_cols_array_2d(),
            color: self.color,
            bump: self.bump,
        }
    }
}
