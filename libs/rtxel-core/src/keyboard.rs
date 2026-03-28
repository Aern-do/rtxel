use std::collections::HashSet;

use bevy_ecs::{observer::On, resource::Resource, system::ResMut, world::World};
pub use winit::keyboard::KeyCode;

use crate::{KeyPress, Plugin};

#[derive(Debug, Default, Clone, Resource)]
pub struct Keyboard {
    pub pressed: HashSet<KeyCode>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
        }
    }

    pub fn press(&mut self, key: KeyCode) {
        self.pressed.insert(key);
    }

    pub fn release(&mut self, key: KeyCode) {
        self.pressed.remove(&key);
    }

    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn clear(&mut self) {
        self.pressed.clear();
    }
}

pub struct KeyboardPlugin;

impl Plugin for KeyboardPlugin {
    fn init(self, world: &mut World) {
        world.init_resource::<Keyboard>();
        world.add_observer(on_key_press);
    }
}

fn on_key_press(key_press: On<KeyPress>, mut keyboard: ResMut<Keyboard>) {
    if key_press.release {
        keyboard.release(key_press.key);
    } else {
        keyboard.press(key_press.key);
    }
}
