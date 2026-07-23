pub(crate) struct Material {
    pub(crate) color: [f32; 4],
    pub(crate) albedo_texture: Option<usize>,
    pub(crate) normal_texture: Option<usize>,
    pub(crate) bump: f32,
}

impl Material {
    pub(crate) fn color(color: [f32; 4]) -> Self {
        Self {
            color,
            albedo_texture: None,
            normal_texture: None,
            bump: 0.0,
        }
    }
}
