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
    DynamicBufferDescriptor, DynamicBufferKind,
    binding::{RStorageBuffer, UniformBuffer},
};
use wgpu::{BindGroupLayout, BufferUsages, ComputePassDescriptor, ComputePipeline};

use crate::{
    Frame, GpuWorld, PipelineSet, RenderStartupSet, UnpackCommandEncoder,
    shared::{GridBinding, MapBinding, PalleteBinding, SharedResources},
};

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
            .add_systems(self.schedule.intern(), extract.in_set(PipelineSet::Extract))
            .add_systems(self.schedule, dispatch.in_set(PipelineSet::Dispatch));
    }
}

type CommandsBinding<const IDX: usize> = Binding<IDX, Compute, RStorageBuffer<u32>>;
type CommandsSizeBinding<const IDX: usize> = Binding<IDX, Compute, UniformBuffer<u32>>;
type OffsetsBinding<const IDX: usize> = Binding<IDX, Compute, RStorageBuffer<u32>>;

type UnpackBindGroup = (
    CommandsBinding<0>,
    CommandsSizeBinding<1>,
    OffsetsBinding<2>,
    MapBinding<3, true>,
    GridBinding<4, true>,
    PalleteBinding<5, true>,
);

const UNPACK_SHADER: &str = include_str!(env!("SHADER_unpack"));
const WORKGROUP_SIZE: u32 = 64;

#[derive(Debug, Resource)]
pub struct UnpackPipeline {
    pub pipeline: ComputePipeline,
    pub bg_layout: BindGroupLayout,
}

impl UnpackPipeline {
    pub fn new(ctx: &Ctx) -> Self {
        let bg_layout = UnpackBindGroup::layout(&ctx);
        let layout = ctx.pipeline_layout(Some("Unpack Pipeline Layout"), &[Some(&bg_layout)]);

        let pipeline = ctx
            .compute_pipeline(BaseComputePipeline {
                layout: &layout,
                shader: &ctx.wgsl_shader(Some("Unpack Shader"), UNPACK_SHADER),
                entry: "main",
            })
            .label("Unpack Compute Pipeline")
            .build();

        Self {
            pipeline,
            bg_layout,
        }
    }
}

#[derive(Debug, Resource)]
pub struct UnpackResources {
    pub command_buffer: DynamicBuffer<u32>,
    pub command_size_buffer: DynamicBuffer<u32>,
    pub offset_buffer: DynamicBuffer<u32>,

    pub command_count: u32,
}

impl UnpackResources {
    pub fn new(ctx: &Ctx) -> Self {
        let command_buffer = DynamicBuffer::new(
            DynamicBufferDescriptor {
                label: Some("Unpack Command Dynamic Buffer".into()),
                usage: BufferUsages::STORAGE,
                kind: DynamicBufferKind::Storage,
            },
            ctx,
        );
        let command_size_buffer = DynamicBuffer::new(
            DynamicBufferDescriptor {
                label: Some("Unpack Command Dynamic Buffer Size".into()),
                usage: BufferUsages::UNIFORM,
                kind: DynamicBufferKind::Uniform,
            },
            ctx,
        );
        let offset_buffer = DynamicBuffer::new(
            DynamicBufferDescriptor {
                label: Some("Unpack Offset Dynamic Buffer".into()),
                usage: BufferUsages::STORAGE,
                kind: DynamicBufferKind::Storage,
            },
            ctx,
        );

        Self {
            command_buffer,
            command_size_buffer,
            offset_buffer,
            command_count: 0,
        }
    }
}

fn init_resources(ctx: Res<Ctx>, mut commands: Commands) {
    commands.insert_resource(UnpackResources::new(&ctx));
}

fn init_pipeline(ctx: Res<Ctx>, mut commands: Commands) {
    commands.insert_resource(UnpackPipeline::new(&ctx));
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
    let mut command_encoder = UnpackCommandEncoder::new();
    for command in commands {
        command_encoder.write_command(command);
    }

    let (commands, offsets) = command_encoder.finish();

    res.command_buffer
        .write(&commands, frame.encoder_mut(), &ctx);
    res.command_size_buffer
        .write(&[len as u32], frame.encoder_mut(), &ctx);
    res.offset_buffer.write(&offsets, frame.encoder_mut(), &ctx);
    res.command_count = len as u32;
}

fn dispatch(
    mut frame: ResMut<Frame>,
    pipeline: Res<UnpackPipeline>,
    resources: Res<UnpackResources>,
    shared_resoucres: Res<SharedResources>,
    ctx: Res<Ctx>,
) {
    if resources.command_count == 0 {
        return;
    }

    let bind_group = UnpackBindGroup::bind_group(
        &ctx,
        &pipeline.bg_layout,
        (
            resources.command_buffer.buffer(),
            resources.command_size_buffer.buffer(),
            resources.offset_buffer.buffer(),
            shared_resoucres.map.buffer(),
            shared_resoucres.grid.buffer(),
            shared_resoucres.palletes.buffer(),
        ),
    );

    let encoder = frame.encoder_mut();
    {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Unpack Compute Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);

        let workgroups = resources.command_count.div_ceil(WORKGROUP_SIZE);
        pass.dispatch_workgroups(workgroups, 1, 1);

        info!(
            "dispatched unpack pass with {} commands",
            resources.command_count
        );
    }
}
