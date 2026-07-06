use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct InputState {
    pressed_keys: HashSet<winit::keyboard::KeyCode>,
}

impl InputState {
    pub(crate) fn is_pressed(&self, key: winit::keyboard::KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub(crate) fn press(&mut self, key: winit::keyboard::KeyCode) {
        self.pressed_keys.insert(key);
    }

    pub(crate) fn release(&mut self, key: winit::keyboard::KeyCode) {
        self.pressed_keys.remove(&key);
    }

    pub(crate) fn clear(&mut self) {
        self.pressed_keys.clear();
    }
}
