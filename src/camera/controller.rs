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
    smoothing: f32,
    smoothed_look: glam::Vec2,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            speed: 6.0,
            sensitivity: 0.002,
            smoothing: 0.5,
            smoothed_look: glam::Vec2::ZERO,
        }
    }
}

impl CameraController {
    pub(crate) fn compute(&mut self, input_state: &mut InputState, dt: f32) -> CameraDisplacement {
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

        let mut raw_look = input_state.take_mouse_pos_delta();
        if !input_state.is_mouse_button_pressed(MouseButton::Right) {
            raw_look = glam::Vec2::ZERO;
            self.smoothed_look = glam::Vec2::ZERO;
        }

        self.smoothed_look += raw_look;
        let look = self.smoothed_look * self.smoothing;
        self.smoothed_look -= look;

        let mouse_scroll_delta = input_state.take_mouse_scroll_delta();

        CameraDisplacement {
            translation: translation.normalize_or_zero() * self.speed * dt,
            fov: mouse_scroll_delta,
            yaw: -look.x * self.sensitivity,
            pitch: -look.y * self.sensitivity,
        }
    }
}
