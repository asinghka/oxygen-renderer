use std::mem::offset_of;

// Ensure uniform values are 16-byte-aligned (std140)
const _: () = assert!(size_of::<RenderSettingsUniform>() == 32);
const _: () = assert!(offset_of!(RenderSettingsUniform, specular_exponent) == 16);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct RenderSettingsUniform {
    ambient_amount: f32,
    diffuse: u32,
    specular: u32,
    specular_strength: f32,
    specular_exponent: f32,
    _pad: [f32; 3],
}

pub(crate) struct RenderSettings {
    pub(crate) ambient: f32,
    pub(crate) diffuse: bool,
    pub(crate) specular: bool,
    pub(crate) specular_strength: f32,
    pub(crate) shininess: f32,
    pub(crate) background: [f32; 3],
    pub(crate) wireframe: bool,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            ambient: 0.1,
            diffuse: true,
            specular: true,
            specular_strength: 0.7,
            shininess: 0.7,
            background: [0.08; 3],
            wireframe: false,
        }
    }
}

impl RenderSettings {
    pub(crate) fn uniform(&self) -> RenderSettingsUniform {
        RenderSettingsUniform {
            ambient_amount: self.ambient,
            diffuse: self.diffuse as u32,
            specular: self.specular as u32,
            specular_strength: self.specular_strength,
            specular_exponent: self.shininess * 256.0,
            _pad: [0.0; 3],
        }
    }
}
