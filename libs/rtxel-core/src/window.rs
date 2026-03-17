use std::sync::Arc;

use bevy_ecs::{event::Event, resource::Resource, world::World};
use winit::window::Window;

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
    }
}
