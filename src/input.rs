use std::collections::HashSet;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

#[derive(Default)]
pub(crate) struct InputState {
    pressed_keys: HashSet<KeyCode>,
    pressed_buttons: HashSet<MouseButton>,
    mouse_pos_delta: glam::Vec2,
    mouse_scroll_delta: f32,
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

    pub(crate) fn add_mouse_pos_delta(&mut self, x: f32, y: f32) {
        self.mouse_pos_delta += glam::vec2(x, y);
    }

    pub(crate) fn take_mouse_pos_delta(&mut self) -> glam::Vec2 {
        let delta = self.mouse_pos_delta;
        self.mouse_pos_delta = glam::Vec2::ZERO;
        delta
    }

    pub(crate) fn add_mouse_scroll_delta(&mut self, x: f32) {
        self.mouse_scroll_delta += x;
    }

    pub(crate) fn take_mouse_scroll_delta(&mut self) -> f32 {
        let delta = self.mouse_scroll_delta;
        self.mouse_scroll_delta = 0.0;
        delta
    }

    pub(crate) fn clear_mouse(&mut self) {
        self.pressed_buttons.clear();
        self.mouse_pos_delta = glam::Vec2::ZERO;
        self.mouse_scroll_delta = 0.0;
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

    pub(crate) fn clear_keys(&mut self) {
        self.pressed_keys.clear();
    }
}
