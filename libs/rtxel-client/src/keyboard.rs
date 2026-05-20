use std::collections::HashSet;

use winit::keyboard::KeyCode;

/// A keyboard input handler
#[derive(Debug, Default, Clone)]
pub struct Keyboard {
    pressed: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
}

impl Keyboard {
    /// Create a keyboard
    pub fn new() -> Self {
        Self::default()
    }

    /// Press a given key
    pub fn press(&mut self, key: KeyCode) {
        if self.pressed.insert(key) {
            self.just_pressed.insert(key);
        }
    }

    /// Release a given key
    pub fn release(&mut self, key: KeyCode) {
        self.pressed.remove(&key);
    }

    /// Check if given key is pressed
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    /// Checked if given key was just pressed.
    ///
    /// Returns `true` only on the frame the key was pressed.
    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    /// Clear keys that were just pressed.
    pub fn clear(&mut self) {
        self.just_pressed.clear();
    }
}
