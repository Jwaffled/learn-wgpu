use std::collections::{HashSet};

use winit::keyboard::KeyCode;

pub struct Input {
    pressed: HashSet<KeyCode>,
    pub mouse_delta: (f32, f32)
}

impl Input {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            mouse_delta: (0.0, 0.0)
        }
    }

    pub fn key_down(&mut self, key: KeyCode) {
        self.pressed.insert(key);
    }

    pub fn key_up(&mut self, key: KeyCode) {
        self.pressed.remove(&key);
    }

    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }
}