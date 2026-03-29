use bevy_ecs::{
    resource::Resource,
    schedule::{InternedScheduleLabel, Schedule, ScheduleLabel},
    world::World,
};

use crate::Plugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Startup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct PreUpdate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Update;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct PostUpdate;

#[derive(Debug, Default, Clone, Resource)]
pub struct Order {
    pub schedules: Vec<InternedScheduleLabel>,
}

impl Order {
    pub fn new(schedules: Vec<InternedScheduleLabel>) -> Self {
        Self { schedules }
    }

    pub fn insert_after(&mut self, after: impl ScheduleLabel, schedule: impl ScheduleLabel) {
        let position = self
            .schedules
            .iter()
            .position(|schedule| *schedule == after.intern())
            .expect("unknown schedule");

        self.schedules.insert(position + 1, schedule.intern());
    }

    pub fn insert_many_after(
        &mut self,
        after: impl ScheduleLabel,
        schedules: &[InternedScheduleLabel],
    ) {
        let pos = self
            .schedules
            .iter()
            .position(|s| *s == after.intern())
            .expect("unknown schedule");

        for (i, schedule) in schedules.iter().enumerate() {
            self.schedules.insert(pos + 1 + i, *schedule);
        }
    }
}

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn init(self, world: &mut World) {
        let schedules = [PreUpdate.intern(), Update.intern(), PostUpdate.intern()];

        for schedule in &schedules {
            world.add_schedule(Schedule::new(*schedule));
        }

        world.insert_resource(Order::new(schedules.to_vec()));
    }
}
