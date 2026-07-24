#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct MaterialUniform {
    color: [f32; 4],
    bump: f32,
}

pub(crate) struct Material {
    pub(crate) color: [f32; 4],
    pub(crate) albedo_texture: Option<usize>,
    pub(crate) normal_texture: Option<usize>,
    pub(crate) bump: f32,
}

impl Material {
    pub(crate) fn uniform(&self) -> MaterialUniform {
        MaterialUniform {
            color: self.color,
            bump: self.bump,
        }
    }
}
