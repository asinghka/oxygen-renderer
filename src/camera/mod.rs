mod controller;
mod uniform;

pub(crate) use controller::*;

use crate::camera::uniform::CameraUniform;
use glam::camera::rh::{proj::directx, view};

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

    pub(crate) fn uniform(&self) -> CameraUniform {
        CameraUniform {
            eye: self.eye.to_array(),
            pad: 0,
            view_projection_matrix: self.build_view_projection_matrix().to_cols_array_2d(),
        }
    }

    fn build_view_projection_matrix(&self) -> glam::Mat4 {
        let target = self.eye + self.forward();

        let view = view::look_at_mat4(self.eye, target, self.up);
        let proj = directx::perspective(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);

        proj * view
    }

    pub(crate) fn displace(&mut self, displacement: CameraDisplacement) {
        self.yaw += displacement.yaw;
        self.pitch = (self.pitch + displacement.pitch).clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        self.fovy = (self.fovy - displacement.fov).clamp(10.0_f32, 120.0_f32);

        self.eye += self.basis() * displacement.translation;
    }

    fn forward(&self) -> glam::Vec3 {
        glam::Vec3::new(-self.yaw.sin() * self.pitch.cos(), self.pitch.sin(), -self.yaw.cos() * self.pitch.cos())
    }

    fn basis(&self) -> glam::Mat3 {
        let forward = self.forward();
        glam::Mat3::from_cols(forward.cross(self.up), self.up, forward)
    }

    pub(crate) fn update_aspect_ratio(&mut self, width: f32, height: f32) {
        if height == 0.0 {
            return;
        } else {
            self.aspect = width / height;
        }
    }
}
