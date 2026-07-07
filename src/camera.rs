use crate::input::InputState;
use glam::camera::rh::{proj::directx, view};
use winit::event::MouseButton;
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
    pub(crate) yaw: f32,
    pub(crate) pitch: f32,
    pub(crate) up: glam::Vec3,
    pub(crate) aspect: f32,
    pub(crate) fovy: f32,
    pub(crate) znear: f32,
    pub(crate) zfar: f32,
}

pub(crate) struct Camera {
    eye: glam::Vec3,
    yaw: f32,
    pitch: f32,
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
            yaw: camera_descriptor.yaw,
            pitch: camera_descriptor.pitch,
            up: camera_descriptor.up,
            aspect: camera_descriptor.aspect,
            fovy: camera_descriptor.fovy,
            znear: camera_descriptor.znear,
            zfar: camera_descriptor.zfar,
        }
    }

    pub(crate) fn build_view_projection_matrix(&self) -> glam::Mat4 {
        let target = self.eye + self.forward();

        let view = view::look_at_mat4(self.eye, target, self.up);
        let proj = directx::perspective(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);

        proj * view
    }

    pub(crate) fn displace(&mut self, displacement: CameraDisplacement) {
        self.yaw += displacement.yaw;
        self.pitch = (self.pitch + displacement.pitch).clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());

        self.eye += self.basis() * displacement.translation;
    }

    fn forward(&self) -> glam::Vec3 {
        glam::Vec3::new(-self.yaw.sin() * self.pitch.cos(), self.pitch.sin(), -self.yaw.cos() * self.pitch.cos())
    }

    fn basis(&self) -> glam::Mat3 {
        let forward = self.forward();
        glam::Mat3::from_cols(forward.cross(self.up), self.up, forward)
    }

    pub(crate) fn update_aspect_ratio(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}

pub(crate) struct CameraDisplacement {
    translation: glam::Vec3,
    yaw: f32,
    pitch: f32,
}

pub(crate) struct CameraController {
    speed: f32,
    sensitivity: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            speed: 6.0,
            sensitivity: 0.004,
        }
    }
}

impl CameraController {
    pub(crate) fn compute(&self, input_state: &mut InputState, dt: f32) -> CameraDisplacement {
        let mut translation = glam::Vec3::ZERO;

        if input_state.is_key_pressed(KeyCode::KeyD) {
            translation += glam::Vec3::X;
        }
        if input_state.is_key_pressed(KeyCode::KeyA) {
            translation -= glam::Vec3::X;
        }
        if input_state.is_key_pressed(KeyCode::KeyQ) {
            translation += glam::Vec3::Y;
        }
        if input_state.is_key_pressed(KeyCode::KeyE) {
            translation -= glam::Vec3::Y;
        }
        if input_state.is_key_pressed(KeyCode::KeyW) {
            translation += glam::Vec3::Z;
        }
        if input_state.is_key_pressed(KeyCode::KeyS) {
            translation -= glam::Vec3::Z;
        }

        let mut mouse_delta = input_state.take_mouse_delta();
        if !input_state.is_mouse_button_pressed(MouseButton::Right) {
            mouse_delta = glam::Vec2::ZERO;
        }

        CameraDisplacement {
            translation: translation.normalize_or_zero() * self.speed * dt,
            yaw: -mouse_delta.x * self.sensitivity,
            pitch: -mouse_delta.y * self.sensitivity,
        }
    }
}
