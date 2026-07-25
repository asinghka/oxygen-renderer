use crate::scene::Vertex;
use std::mem::offset_of;

const _: () = assert!(offset_of!(GridPrimitiveUniform, color) == 64);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GridPrimitiveUniform {
    model: [[f32; 4]; 4],
    color: [f32; 4],
}

pub(crate) struct GridPrimitive {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
    pub(crate) model: glam::Mat4,
    pub(crate) color: wgpu::Color,
}

impl GridPrimitive {
    pub(crate) fn grid(size: f32, divisions: u32) -> Self {
        let step = size / divisions as f32;
        let offsets = (0..=divisions).map(|n| -size / 2.0 + n as f32 * step);

        Self::new(
            size,
            offsets,
            wgpu::Color {
                r: 0.25,
                g: 0.25,
                b: 0.25,
                a: 1.0,
            },
        )
    }

    pub(crate) fn subgrid(size: f32, divisions: u32) -> Self {
        let step = size / divisions as f32;
        let offsets = (0..divisions).map(|n| -size / 2.0 + (n as f32 + 0.5) * step);

        Self::new(
            size,
            offsets,
            wgpu::Color {
                r: 0.1,
                g: 0.1,
                b: 0.1,
                a: 1.0,
            },
        )
    }

    pub(crate) fn new(size: f32, offsets: impl Iterator<Item = f32>, color: wgpu::Color) -> Self {
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
        }
    }

    pub(crate) fn uniform(&self) -> GridPrimitiveUniform {
        GridPrimitiveUniform {
            model: self.model.to_cols_array_2d(),
            color: [self.color.r as f32, self.color.g as f32, self.color.b as f32, self.color.a as f32],
        }
    }
}
