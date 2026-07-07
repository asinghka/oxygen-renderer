#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}

impl Vertex {
    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

pub(crate) const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [1.0, 0.5, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [1.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [0.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [0.0, 0.3, 1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [0.6, 0.0, 1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [1.0, 0.0, 0.8, 1.0],
    },
];

pub(crate) const INDICES: &[u16] = &[
    4, 5, 6, 4, 6, 7, 1, 0, 3, 1, 3, 2, 1, 2, 6, 1, 6, 5, 0, 7, 3, 0, 4, 7, 3, 6, 2, 3, 7, 6, 0, 1, 5, 0, 5, 4,
];
