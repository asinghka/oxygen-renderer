use glam::camera::rh::proj::directx;
use glam::camera::rh::view;
use std::f32::consts::PI;
use std::mem::offset_of;

const _: () = assert!(size_of::<LightUniform>() == 80);
const _: () = assert!(offset_of!(LightUniform, view_orthographic_matrix) == 16);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightUniform {
    direction: [f32; 3],
    _pad: f32,
    view_orthographic_matrix: [[f32; 4]; 4],
}

pub(crate) struct Light {
    pub(crate) azimuth: f32,
    pub(crate) elevation: f32,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            azimuth: PI,
            elevation: PI / 3.0,
        }
    }
}

impl Light {
    pub(crate) fn uniform(&self) -> LightUniform {
        let direction = self.direction();

        LightUniform {
            direction: direction.to_array(),
            _pad: 0.0,
            view_orthographic_matrix: self.view_orthographic_matrix(20.0, 10.0).to_cols_array_2d(),
        }
    }

    fn view_orthographic_matrix(&self, d: f32, r: f32) -> glam::Mat4 {
        let up = glam::Vec3::Y;
        let eye = d * self.direction();
        let target = glam::Vec3::ZERO;

        let view = view::look_at_mat4(eye, target, up);
        let ortho = directx::orthographic(-r, r, -r, r, d - r, d + r);
        ortho * view
    }

    fn direction(&self) -> glam::Vec3 {
        glam::Vec3::new(
            -self.azimuth.sin() * self.elevation.cos(),
            self.elevation.sin(),
            -self.azimuth.cos() * self.elevation.cos(),
        )
    }
}
