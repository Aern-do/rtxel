pub mod camera;
pub mod render;
pub mod world;
use bevy_ecs::{component::Component, world::World};
pub use camera::*;
pub use render::*;
use rtxel_core::{Plugin, WorldExt};
pub use world::*;

#[derive(Debug, Clone, Copy, Component)]
pub struct Player;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn init(self, world: &mut World) {
        world.add_plugin(RenderPlugin);
        world.add_plugin(CameraPlugin);
        world.add_plugin(WorldPlugin);
    }
}
