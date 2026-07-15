#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightUniform {
    direction: [f32; 3],
}

#[derive(Default)]
pub(crate) struct Light {
    azimuth: f32,
    elevation: f32,
}

impl Light {
    pub(crate) fn uniform(&self) -> LightUniform {
        let direction = [
            -self.azimuth.sin() * self.elevation.cos(),
            self.elevation.sin(),
            -self.azimuth.cos() * self.elevation.cos(),
        ];

        LightUniform { direction }
    }
}
