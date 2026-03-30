use bevy_ecs::{
    resource::Resource,
    schedule::{IntoScheduleConfigs, ScheduleLabel},
    system::{Commands, Res, ResMut},
    world::World,
};
use log::info;
use rtxel_core::{Plugin, Startup, WorldExt};
use rtxel_gpu::{
    AsBindGroup, BaseComputePipeline, Binding, Compute, Ctx, DynamicBuffer,
    DynamicBufferDescriptor, DynamicBufferKind, RWStorageBuffer,
    binding::{RStorageBuffer, UniformBuffer},
};
use wgpu::{BindGroup, BindGroupLayout, BufferUsages, ComputePassDescriptor, ComputePipeline};

use crate::{
    Frame, GpuBrickMap, GpuWorld, PipelineSet, RenderStartupSet, UnpackCommand,
    shared::SharedResources,
};

type UnpackBindGroup = (
    Binding<0, Compute, RStorageBuffer<UnpackCommand>>,
    Binding<1, Compute, UniformBuffer<u32>>,
);

type SharedBindGroup = (
    Binding<0, Compute, RWStorageBuffer<u32>>,
    Binding<1, Compute, RWStorageBuffer<GpuBrickMap>>,
);

const UNPACK_SHADER: &str = include_str!(env!("SHADER_unpack"));
const WORKGROUP_SIZE: u32 = 64;

pub struct UnpackPipelinePlugin<S> {
    pub schedule: S,
}

impl<S: ScheduleLabel> Plugin for UnpackPipelinePlugin<S> {
    fn init(self, world: &mut World) {
        world
            .add_systems(
                Startup,
                (init_resources, init_pipeline)
                    .chain()
                    .in_set(RenderStartupSet::Resources),
            )
            .add_systems(
                self.schedule.intern(),
                (
                    extract,
                    rebuild.run_if(|resources: Res<UnpackResources>| resources.is_dirty),
                    rebuild_shared.run_if(|shared: Res<SharedResources>| shared.is_dirty),
                )
                    .chain()
                    .in_set(PipelineSet::Extract),
            )
            .add_systems(self.schedule, dispatch.in_set(PipelineSet::Dispatch));
    }
}

#[derive(Debug, Resource)]
pub struct UnpackPipeline {
    pub pipeline: ComputePipeline,
}

impl UnpackPipeline {
    pub fn new(ctx: &Ctx, resources: &UnpackResources) -> Self {
        let layout = ctx.pipeline_layout(
            Some("Unpack Pipeline Layout"),
            &[
                Some(&resources.bg_layout),
                Some(&resources.shared_bg_layout),
            ],
        );

        let pipeline = ctx
            .compute_pipeline(BaseComputePipeline {
                layout: &layout,
                shader: &ctx.wgsl_shader(Some("Unpack Shader"), UNPACK_SHADER),
                entry: "main",
            })
            .label("Unpack Compute Pipeline")
            .build();

        Self { pipeline }
    }
}

#[derive(Debug, Resource)]
pub struct UnpackResources {
    pub buffer: DynamicBuffer<UnpackCommand>,
    pub buffer_size: DynamicBuffer<u32>,
    pub command_count: u32,

    pub bg: BindGroup,
    pub bg_layout: BindGroupLayout,
    pub shared_bg: BindGroup,
    pub shared_bg_layout: BindGroupLayout,

    pub is_dirty: bool,
}

impl UnpackResources {
    pub fn new(ctx: &Ctx, shared: &SharedResources) -> Self {
        let buffer = DynamicBuffer::new(
            DynamicBufferDescriptor {
                label: Some("Unpack Dynamic Buffer".into()),
                usage: BufferUsages::STORAGE,
                kind: DynamicBufferKind::Storage,
            },
            ctx,
        );
        let buffer_size = DynamicBuffer::new(
            DynamicBufferDescriptor {
                label: Some("Unpack Dynamic Buffer Size".into()),
                usage: BufferUsages::UNIFORM,
                kind: DynamicBufferKind::Uniform,
            },
            ctx,
        );

        let bg_layout = UnpackBindGroup::layout(ctx);
        let bg = Self::create_bind_group(ctx, &bg_layout, &buffer, &buffer_size);

        let shared_bg_layout = SharedBindGroup::layout(ctx);
        let shared_bg = Self::create_shared_bind_group(ctx, &shared_bg_layout, shared);

        Self {
            buffer,
            buffer_size,
            bg_layout,
            bg,
            shared_bg,
            shared_bg_layout,
            command_count: 0,
            is_dirty: false,
        }
    }

    fn create_bind_group(
        ctx: &Ctx,
        layout: &BindGroupLayout,
        buffer: &DynamicBuffer<UnpackCommand>,
        buffer_size: &DynamicBuffer<u32>,
    ) -> BindGroup {
        UnpackBindGroup::bind_group(ctx, layout, (buffer.buffer(), buffer_size.buffer()))
    }

    fn create_shared_bind_group(
        ctx: &Ctx,
        layout: &BindGroupLayout,
        shared: &SharedResources,
    ) -> BindGroup {
        SharedBindGroup::bind_group(ctx, layout, (shared.grid.buffer(), shared.map.buffer()))
    }

    pub fn rebuild_bind_group(&mut self, ctx: &Ctx) {
        self.bg = Self::create_bind_group(ctx, &self.bg_layout, &self.buffer, &self.buffer_size);
    }

    pub fn rebuild_shared_bind_group(&mut self, ctx: &Ctx, shared: &SharedResources) {
        self.shared_bg = Self::create_shared_bind_group(ctx, &self.shared_bg_layout, shared);
    }
}

fn init_resources(ctx: Res<Ctx>, shared: Res<SharedResources>, mut commands: Commands) {
    commands.insert_resource(UnpackResources::new(&ctx, &shared));
}

fn init_pipeline(ctx: Res<Ctx>, resources: Res<UnpackResources>, mut commands: Commands) {
    commands.insert_resource(UnpackPipeline::new(&ctx, &resources));
}

fn extract(
    ctx: Res<Ctx>,
    mut frame: ResMut<Frame>,
    mut world: ResMut<GpuWorld>,
    mut res: ResMut<UnpackResources>,
) {
    let (len, commands) = world.drain_commands();
    if len == 0 {
        res.command_count = 0;
        return;
    }

    let encoder = frame.encoder_mut();

    res.command_count = len as u32;
    res.buffer.write_iter(commands, encoder, &ctx);
    res.buffer_size.write(&[len as u32], encoder, &ctx);
    res.is_dirty = true;
}

fn rebuild(ctx: Res<Ctx>, mut res: ResMut<UnpackResources>) {
    res.rebuild_bind_group(&ctx);
    res.is_dirty = false;
    info!("unpack bind group rebuilt");
}

fn rebuild_shared(ctx: Res<Ctx>, shared: Res<SharedResources>, mut res: ResMut<UnpackResources>) {
    res.rebuild_shared_bind_group(&ctx, &shared);
    info!("unpack shared bind group rebuilt");
}

fn dispatch(
    mut frame: ResMut<Frame>,
    pipeline: Res<UnpackPipeline>,
    resources: Res<UnpackResources>,
) {
    if resources.command_count == 0 {
        return;
    }

    let encoder = frame.encoder_mut();
    {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Unpack Compute Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline.pipeline);
        pass.set_bind_group(0, &resources.bg, &[]);
        pass.set_bind_group(1, &resources.shared_bg, &[]);

        let workgroups = resources.command_count.div_ceil(WORKGROUP_SIZE);
        pass.dispatch_workgroups(workgroups, 1, 1);

        info!(
            "dispatched unpack pass with {} commands",
            resources.command_count
        );
    }
}
