pub mod camera;
pub use camera::*;

use bevy_ecs::{component::Component, world::World};
use rtxel_core::{Plugin, WorldExt};

#[derive(Debug, Clone, Copy, Component)]
pub struct Player;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn init(self, world: &mut World) {
        world.add_plugin(CameraPlugin);
    }
}
