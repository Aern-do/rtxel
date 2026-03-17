use bevy_ecs::{
    schedule::{IntoScheduleConfigs, ScheduleLabel, Schedules},
    system::ScheduleSystem,
    world::World,
};

use crate::Plugin;

pub trait WorldExt {
    fn add_plugin(&mut self, plugin: impl Plugin);
    fn add_systems<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    );
}

impl WorldExt for World {
    fn add_plugin(&mut self, plugin: impl Plugin) {
        plugin.init(self);
    }

    fn add_systems<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) {
        self.get_resource_or_init::<Schedules>()
            .add_systems(schedule, systems);
    }
}
