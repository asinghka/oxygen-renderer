use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct InputHandler {
    pressed_keys: HashSet<winit::keyboard::KeyCode>,
}

impl InputHandler {
    pub(crate) fn contains(&self, key: winit::keyboard::KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub(crate) fn key_pressed(&mut self, key: winit::keyboard::KeyCode) {
        self.pressed_keys.insert(key);
    }

    pub(crate) fn key_released(&mut self, key: winit::keyboard::KeyCode) {
        self.pressed_keys.remove(&key);
    }

    pub(crate) fn clear(&mut self) {
        self.pressed_keys.clear();
    }
}
