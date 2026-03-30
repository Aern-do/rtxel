use bevy_ecs::{
    schedule::{InternedScheduleLabel, IntoScheduleConfigs, ScheduleLabel, SystemSet},
    world::World,
};
use rtxel_core::{Order, Plugin, WorldExt};

use crate::{
    draw::DrawPipelinePlugin, present::PresentPipelinePlugin, shared::SharedPipelinePlugin,
    unpack::UnpackPipelinePlugin,
};

pub mod draw;
pub mod present;
pub mod shared;
pub mod unpack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Shared;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Unpack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Draw;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Present;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum PipelineSet {
    Extract,
    Dispatch,
}

#[derive(Debug, Default)]
pub struct PipelinePlugin<B, C> {
    pub begin_frame: B,
    pub clean: C,
}

impl<B: ScheduleLabel, C: ScheduleLabel> Plugin for PipelinePlugin<B, C> {
    fn init(self, world: &mut World) {
        let passes = [
            Shared.intern(),
            Unpack.intern(),
            Draw.intern(),
            Present.intern(),
        ];

        add_pases(self.begin_frame, world, &passes);

        world.add_plugin(SharedPipelinePlugin {
            schedule: Shared,
            clean: self.clean.intern(),
        });
        world.add_plugin(UnpackPipelinePlugin { schedule: Unpack });
        world.add_plugin(DrawPipelinePlugin { schedule: Draw });
        world.add_plugin(PresentPipelinePlugin { schedule: Present });
    }
}

fn add_pases<B: ScheduleLabel>(
    begin_frame: B,
    world: &mut World,
    passes: &[InternedScheduleLabel],
) {
    let mut order = world.resource_mut::<Order>();
    order.insert_many_after(begin_frame, passes);

    for pass in passes {
        world.configure_sets(*pass, (PipelineSet::Extract, PipelineSet::Dispatch).chain());
    }
}
