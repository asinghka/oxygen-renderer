use glam::camera::rh::{proj::directx, view};
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CameraUniform {
    view_projection_matrix: [[f32; 4]; 4],
}

impl CameraUniform {
    pub(crate) fn new() -> Self {
        Self {
            view_projection_matrix: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub(crate) fn update_view_projection_matrix(&mut self, camera: &Camera) {
        self.view_projection_matrix = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}

pub(crate) struct Camera {
    pub(crate) eye: Vec3,
    pub(crate) target: Vec3,
    pub(crate) up: Vec3,
    pub(crate) aspect: f32,
    pub(crate) fovy: f32,
    pub(crate) znear: f32,
    pub(crate) zfar: f32,
}

impl Camera {
    pub(crate) fn build_view_projection_matrix(&self) -> Mat4 {
        let view = view::look_at_mat4(self.eye, self.target, self.up);
        let proj = directx::perspective(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);

        proj * view
    }

    pub(crate) fn update_aspect_ratio(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}
