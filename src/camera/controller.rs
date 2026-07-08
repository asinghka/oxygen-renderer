use crate::app::InputState;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

pub(crate) struct CameraDisplacement {
    pub(crate) translation: glam::Vec3,
    pub(crate) fov: f32,
    pub(crate) yaw: f32,
    pub(crate) pitch: f32,
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

        let mut mouse_pos_delta = input_state.take_mouse_pos_delta();
        if !input_state.is_mouse_button_pressed(MouseButton::Right) {
            mouse_pos_delta = glam::Vec2::ZERO;
        }

        let mouse_scroll_delta = input_state.take_mouse_scroll_delta();

        CameraDisplacement {
            translation: translation.normalize_or_zero() * self.speed * dt,
            fov: mouse_scroll_delta,
            yaw: -mouse_pos_delta.x * self.sensitivity,
            pitch: -mouse_pos_delta.y * self.sensitivity,
        }
    }
}
