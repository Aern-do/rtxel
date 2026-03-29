use bevy_ecs::{
    resource::Resource,
    schedule::{IntoScheduleConfigs, ScheduleLabel},
    system::{Commands, Res, ResMut},
    world::World,
};
use rtxel_core::{Plugin, Startup, WorldExt};
use rtxel_gpu::{Ctx, DynamicBuffer, DynamicBufferDescriptor, DynamicBufferKind};
use wgpu::{BufferUsages, Texture, TextureFormat, TextureUsages};

use crate::{BrickGrid, Frame, GpuBrickMap, GpuWorld, PipelineSet, RenderStartupSet};

#[derive(Debug, Resource)]
pub struct SharedResources {
    pub grid: DynamicBuffer<u32>,
    pub map: DynamicBuffer<GpuBrickMap>,
    pub out_texture: Texture,

    pub is_dirty: bool,
}

impl SharedResources {
    pub fn new(ctx: &Ctx) -> Self {
        let (width, height) = ctx.size();

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
            out_texture: ctx.texture(
                Some("Output Texture"),
                width,
                height,
                TextureFormat::Rgba32Float,
                TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            ),
            is_dirty: false,
        }
    }
}

pub struct SharedPipelinePlugin<S, C> {
    pub schedule: S,
    pub clean: C,
}

impl<S: ScheduleLabel, C: ScheduleLabel> Plugin for SharedPipelinePlugin<S, C> {
    fn init(self, world: &mut World) {
        world
            .add_systems(Startup, init.in_set(RenderStartupSet::SharedResources))
            .add_systems(self.schedule, extract.in_set(PipelineSet::Extract))
            .add_systems(self.clean, clean);
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

fn clean(mut resources: ResMut<SharedResources>) {
    resources.is_dirty = false
}
