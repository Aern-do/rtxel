use std::sync::Arc;

use bevy_ecs::{event::Event, resource::Resource, world::World};
use winit::{keyboard::KeyCode, window::Window};

use crate::Plugin;

#[derive(Debug, Resource)]
pub struct WindowHandle {
    pub handle: Arc<Window>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Event)]
pub struct RedrawRequested;

#[derive(Debug, Clone, Copy, PartialEq, Event)]
pub struct MouseMotion {
    pub delta_x: f64,
    pub delta_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Event)]
pub struct KeyPress {
    pub key: KeyCode,
    pub release: bool,
}

#[derive(Debug, Default, Resource, Clone, Copy)]
pub struct DeltaTime {
    pub seconds: f32,
}

impl DeltaTime {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
pub struct WindowPlugin {
    pub window: Window,
}

impl Plugin for WindowPlugin {
    fn init(self, world: &mut World) {
        world.register_event_key::<RedrawRequested>();
        world.insert_resource(WindowHandle {
            handle: Arc::new(self.window),
        });
        world.insert_resource(DeltaTime::new());
    }
}
