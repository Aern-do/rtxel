pub mod keyboard;
pub mod mouse;
pub mod schedule;
pub mod start;
pub mod window;
pub mod world_ext;
use bevy_ecs::world::World;
pub use keyboard::*;
pub use mouse::*;
pub use schedule::*;
pub use start::*;
pub use window::*;
use winit::window::Window;
pub use world_ext::*;

pub trait Plugin {
    fn init(self, world: &mut World);
}

pub struct CorePlugin {
    pub window: Window,
}

impl Plugin for CorePlugin {
    fn init(self, world: &mut World) {
        world.add_plugin(SchedulePlugin);
        world.add_plugin(WindowPlugin {
            window: self.window,
        });
        world.add_plugin(KeyboardPlugin);
        world.add_plugin(MousePlugin);
    }
}
