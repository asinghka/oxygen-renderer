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
        position: [-0.0868241, 0.49240386, 0.0],
        color: [1.0, 0.2, 0.3, 1.0],
    },
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [1.0, 0.65, 0.1, 1.0],
    },
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.2, 0.85, 0.35, 1.0],
    },
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.1, 0.55, 1.0, 1.0],
    },
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.75, 0.25, 1.0, 1.0],
    },
];

pub(crate) const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
