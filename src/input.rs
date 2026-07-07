use std::collections::HashSet;
use winit::keyboard::KeyCode;

#[derive(Default)]
pub(crate) struct InputState {
    pressed_keys: HashSet<KeyCode>,
    mouse_delta: glam::Vec2,
}

impl InputState {
    pub(crate) fn add_mouse_delta(&mut self, x: f32, y: f32) {
        self.mouse_delta += glam::vec2(x, y);
    }

    pub(crate) fn take_mouse_delta(&mut self) -> glam::Vec2 {
        let delta = self.mouse_delta;
        self.mouse_delta = glam::Vec2::ZERO;
        delta
    }

    pub(crate) fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub(crate) fn press(&mut self, key: KeyCode) {
        self.pressed_keys.insert(key);
    }

    pub(crate) fn release(&mut self, key: KeyCode) {
        self.pressed_keys.remove(&key);
    }

    pub(crate) fn clear(&mut self) {
        self.pressed_keys.clear();
    }
}
