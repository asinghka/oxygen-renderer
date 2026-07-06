use crate::input::InputState;
use glam::camera::rh::{proj::directx, view};
use winit::keyboard::KeyCode;

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

    pub(crate) fn displace(&mut self, displacement: CameraDisplacement) {
        self.eye += displacement.translation;
        self.target += displacement.translation;
    }

    pub(crate) fn update_aspect_ratio(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}

pub(crate) struct CameraDisplacement {
    translation: glam::Vec3,
}

pub(crate) struct CameraController {
    speed: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self { speed: 6.0 }
    }
}

impl CameraController {
    pub(crate) fn compute(&self, input_state: &InputState, dt: f32) -> CameraDisplacement {
        let mut translation = glam::Vec3::ZERO;

        if input_state.is_pressed(KeyCode::KeyA) {
            translation -= glam::Vec3::X;
        }
        if input_state.is_pressed(KeyCode::KeyD) {
            translation += glam::Vec3::X;
        }
        if input_state.is_pressed(KeyCode::KeyQ) {
            translation += glam::Vec3::Y;
        }
        if input_state.is_pressed(KeyCode::KeyE) {
            translation -= glam::Vec3::Y;
        }
        if input_state.is_pressed(KeyCode::KeyW) {
            translation -= glam::Vec3::Z;
        }
        if input_state.is_pressed(KeyCode::KeyS) {
            translation += glam::Vec3::Z;
        }

        CameraDisplacement {
            translation: translation.normalize_or_zero() * self.speed * dt,
        }
    }
}
