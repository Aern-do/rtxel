use bevy_ecs::{
    resource::Resource,
    schedule::IntoScheduleConfigs,
    system::{Commands, Res, ResMut},
    world::World,
};
use rtxel_core::{Plugin, Startup, WorldExt};
use rtxel_gpu::{Ctx, DynamicBuffer, DynamicBufferDescriptor, DynamicBufferKind};
use wgpu::BufferUsages;

use crate::{
    BrickGrid, Frame, GpuBrickMap, GpuWorld, PipelineSet, Render, RenderSet, RenderStartupSet,
};

#[derive(Debug, Resource)]
pub struct SharedResources {
    pub grid: DynamicBuffer<u32>,
    pub map: DynamicBuffer<GpuBrickMap>,

    pub is_dirty: bool,
}

impl SharedResources {
    pub fn new(ctx: &Ctx) -> Self {
        Self {
            grid: DynamicBuffer::new(
                DynamicBufferDescriptor {
                    label: Some("Grid Dynamic Buffer".into()),
                    usage: BufferUsages::STORAGE,
                    kind: DynamicBufferKind::Storage,
                },
                ctx,
            ),
            map: DynamicBuffer::new(
                DynamicBufferDescriptor {
                    label: Some("Map Dynamic Buffer".into()),
                    usage: BufferUsages::STORAGE,
                    kind: DynamicBufferKind::Storage,
                },
                ctx,
            ),
            is_dirty: false,
        }
    }
}

pub struct SharedPipelinePlugin;

impl Plugin for SharedPipelinePlugin {
    fn init(self, world: &mut World) {
        world
            .add_systems(
                Startup,
                init.in_set(RenderStartupSet::Resources)
                    .in_set(PipelineSet::Shared),
            )
            .add_systems(
                Render,
                (
                    extract
                        .in_set(RenderSet::Extract)
                        .in_set(PipelineSet::Shared),
                    clear_dirty.in_set(RenderSet::EndFrame),
                ),
            );
    }
}

fn init(ctx: Res<Ctx>, mut commands: Commands) {
    commands.insert_resource(SharedResources::new(&ctx));
}

fn extract(
    world: Res<GpuWorld>,
    grid: Res<BrickGrid>,
    mut resources: ResMut<SharedResources>,
    mut frame: ResMut<Frame>,
    ctx: Res<Ctx>,
) {
    if resources
        .map
        .ensure_capacity(world.map_capacity(), frame.encoder_mut(), &ctx)
    {
        resources.is_dirty = true;
    }

    if resources
        .grid
        .ensure_capacity(grid.size as u64, frame.encoder_mut(), &ctx)
    {
        resources.grid.mark_fully_used();
        resources.is_dirty = true;
    }
}

fn clear_dirty(mut resources: ResMut<SharedResources>) {
    resources.is_dirty = false
}
