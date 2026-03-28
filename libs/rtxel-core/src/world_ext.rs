use bevy_ecs::{
    schedule::{InternedSystemSet, IntoScheduleConfigs, ScheduleLabel, Schedules},
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
    ) -> &mut Self;

    fn configure_sets<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        sets: impl IntoScheduleConfigs<InternedSystemSet, M>,
    ) -> &mut Self;
}

impl WorldExt for World {
    fn add_plugin(&mut self, plugin: impl Plugin) {
        plugin.init(self);
    }

    fn add_systems<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.get_resource_or_init::<Schedules>()
            .add_systems(schedule, systems);

        self
    }

    fn configure_sets<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        sets: impl IntoScheduleConfigs<InternedSystemSet, M>,
    ) -> &mut Self {
        let mut schedules = self.resource_mut::<Schedules>();
        schedules.configure_sets(schedule, sets);
        self
    }
}
