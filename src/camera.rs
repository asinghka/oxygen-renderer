use glam::camera::rh::{proj::directx, view};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CameraUniform {
    view_projection_matrix: [[f32; 4]; 4],
}

impl CameraUniform {
    pub(crate) fn new() -> Self {
        Self {
            view_projection_matrix: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub(crate) fn update_view_projection_matrix(&mut self, camera: &Camera) {
        self.view_projection_matrix = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}

pub(crate) struct CameraDescriptor {
    pub(crate) eye: glam::Vec3,
    pub(crate) target: glam::Vec3,
    pub(crate) up: glam::Vec3,
    pub(crate) aspect: f32,
    pub(crate) fovy: f32,
    pub(crate) znear: f32,
    pub(crate) zfar: f32,
}

pub(crate) struct Camera {
    eye: glam::Vec3,
    target: glam::Vec3,
    up: glam::Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub(crate) fn new(camera_descriptor: &CameraDescriptor) -> Self {
        Self {
            eye: camera_descriptor.eye,
            target: camera_descriptor.target,
            up: camera_descriptor.up,
            aspect: camera_descriptor.aspect,
            fovy: camera_descriptor.fovy,
            znear: camera_descriptor.znear,
            zfar: camera_descriptor.zfar,
        }
    }

    pub(crate) fn build_view_projection_matrix(&self) -> glam::Mat4 {
        let view = view::look_at_mat4(self.eye, self.target, self.up);
        let proj = directx::perspective(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);

        proj * view
    }

    pub(crate) fn update(&mut self, direction: glam::Vec3) {
        self.eye += direction * 0.1;
        self.target += direction * 0.1;
    }

    pub(crate) fn update_aspect_ratio(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}
