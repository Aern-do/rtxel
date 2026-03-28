use bevy_ecs::{
    schedule::{IntoScheduleConfigs, SystemSet},
    world::World,
};
use rtxel_core::{Plugin, Startup, WorldExt};

use crate::{Render, shared::SharedPipelinePlugin, unpack::UnpackPipelinePlugin};

pub mod shared;
pub mod unpack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum PipelineSet {
    Shared,
    Unpack,
    Compute,
    Draw,
}

pub struct PipelinePlugin;

impl Plugin for PipelinePlugin {
    fn init(self, world: &mut World) {
        world
            .configure_sets(
                Startup,
                (
                    PipelineSet::Shared,
                    PipelineSet::Unpack,
                    PipelineSet::Compute,
                    PipelineSet::Draw,
                )
                    .chain(),
            )
            .configure_sets(
                Render,
                (
                    PipelineSet::Shared,
                    PipelineSet::Unpack,
                    PipelineSet::Compute,
                    PipelineSet::Draw,
                )
                    .chain(),
            );

        world.add_plugin(SharedPipelinePlugin);
        world.add_plugin(UnpackPipelinePlugin);
    }
}
