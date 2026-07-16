#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightUniform {
    direction: [f32; 3],
    _pad: f32,
}

#[derive(Default)]
pub(crate) struct Light {
    pub(crate) azimuth: f32,
    pub(crate) elevation: f32,
}

impl Light {
    pub(crate) fn uniform(&self) -> LightUniform {
        let direction = [
            -self.azimuth.sin() * self.elevation.cos(),
            self.elevation.sin(),
            -self.azimuth.cos() * self.elevation.cos(),
        ];

        LightUniform { direction, _pad: 0.0 }
    }
}
