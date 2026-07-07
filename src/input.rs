use std::collections::HashSet;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

#[derive(Default)]
pub(crate) struct InputState {
    pressed_keys: HashSet<KeyCode>,
    pressed_buttons: HashSet<MouseButton>,
    mouse_delta: glam::Vec2,
}

impl InputState {
    pub(crate) fn mouse_press(&mut self, button: MouseButton) {
        self.pressed_buttons.insert(button);
    }

    pub(crate) fn mouse_release(&mut self, button: MouseButton) {
        self.pressed_buttons.remove(&button);
    }

    pub(crate) fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    pub(crate) fn add_mouse_delta(&mut self, x: f32, y: f32) {
        self.mouse_delta += glam::vec2(x, y);
    }

    pub(crate) fn take_mouse_delta(&mut self) -> glam::Vec2 {
        let delta = self.mouse_delta;
        self.mouse_delta = glam::Vec2::ZERO;
        delta
    }

    pub(crate) fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub(crate) fn key_press(&mut self, key: KeyCode) {
        self.pressed_keys.insert(key);
    }

    pub(crate) fn key_release(&mut self, key: KeyCode) {
        self.pressed_keys.remove(&key);
    }

    pub(crate) fn clear(&mut self) {
        self.pressed_keys.clear();
    }
}
