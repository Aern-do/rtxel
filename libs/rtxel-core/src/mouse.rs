use std::collections::HashSet;

use bevy_ecs::{observer::On, resource::Resource, system::ResMut, world::World};
pub use winit::event::MouseButton;

use crate::{MousePress, Plugin, PostUpdate, WorldExt};

#[derive(Debug, Default, Clone, Resource)]
pub struct Mouse {
    pub pressed: HashSet<MouseButton>,
    pub just_pressed: HashSet<MouseButton>,
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
        }
    }

    pub fn press(&mut self, button: MouseButton) {
        if self.pressed.insert(button) {
            self.just_pressed.insert(button);
        }
    }

    pub fn release(&mut self, button: MouseButton) {
        self.pressed.remove(&button);
    }

    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.pressed.contains(&button)
    }

    pub fn just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed.contains(&button)
    }

    pub fn clear_just_pressed(&mut self) {
        self.just_pressed.clear();
    }
}

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn init(self, world: &mut World) {
        world.init_resource::<Mouse>();
        world.add_observer(on_mouse_press);
        world.add_systems(PostUpdate, clear_just_pressed);
    }
}

fn on_mouse_press(mouse_press: On<MousePress>, mut mouse: ResMut<Mouse>) {
    if mouse_press.release {
        mouse.release(mouse_press.button);
    } else {
        mouse.press(mouse_press.button);
    }
}

fn clear_just_pressed(mut mouse: ResMut<Mouse>) {
    mouse.clear_just_pressed();
}