pub mod schedule;
pub mod start;
pub mod window;
pub mod world_ext;

pub use schedule::*;
pub use start::*;
pub use window::*;
pub use world_ext::*;

use bevy_ecs::world::World;
use winit::window::Window;

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
    }
}
