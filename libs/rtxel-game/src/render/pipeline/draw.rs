use bevy_ecs::{
    query::With,
    resource::Resource,
    schedule::{IntoScheduleConfigs, ScheduleLabel},
    system::{Commands, Query, Res, ResMut},
    world::World,
};
use log::info;
use rtxel_core::{Plugin, Startup, WorldExt};
use rtxel_gpu::{
    AsBindGroup, BaseComputePipeline, Binding, Compute, Ctx, DynamicBuffer,
    DynamicBufferDescriptor, DynamicBufferKind, Rgba32Float, UniformBuffer, WStorageTexture,
    binding::RStorageBuffer,
};
use wgpu::{
    BindGroup, BindGroupLayout, BufferUsages, ComputePassDescriptor, ComputePipeline,
    wgt::TextureViewDescriptor,
};

use crate::{Camera, Frame, PipelineSet, Player, RenderStartupSet, shared::SharedResources};

type DrawBindGroup = Binding<0, Compute, UniformBuffer<Camera>>;

type SharedBindGroup = (
    Binding<0, Compute, RStorageBuffer<u32>>,
    Binding<1, Compute, RStorageBuffer<u32>>,
    Binding<2, Compute, WStorageTexture<Rgba32Float>>,
);

const DRAW_SHADER: &str = include_str!(env!("SHADER_draw"));
const WORKGROUP_SIZE: u32 = 8;

pub struct DrawPipelinePlugin<S> {
    pub schedule: S,
}

impl<S: ScheduleLabel> Plugin for DrawPipelinePlugin<S> {
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
                    extract_camera,
                    rebuild.run_if(|shared: Res<SharedResources>| shared.is_dirty),
                )
                    .in_set(PipelineSet::Extract),
            )
            .add_systems(self.schedule, dispatch.in_set(PipelineSet::Dispatch));
    }
}

#[derive(Debug, Resource)]
pub struct DrawPipeline {
    pub pipeline: ComputePipeline,
}

impl DrawPipeline {
    pub fn new(ctx: &Ctx, resources: &DrawResources) -> Self {
        let layout = ctx.pipeline_layout(
            Some("Draw Pipeline Layout"),
            &[&resources.shared_bg_layout, &resources.bg_layout],
        );

        let pipeline = ctx
            .compute_pipeline(BaseComputePipeline {
                layout: &layout,
                shader: &ctx.wgsl_shader(Some("Draw Shader"), DRAW_SHADER),
                entry: "main",
            })
            .label("Draw Compute Pipeline")
            .build();

        Self { pipeline }
    }
}

#[derive(Debug, Resource)]
pub struct DrawResources {
    pub camera: DynamicBuffer<Camera>,
    pub bg: BindGroup,
    pub bg_layout: BindGroupLayout,

    pub shared_bg: BindGroup,
    pub shared_bg_layout: BindGroupLayout,

    pub width: u32,
    pub height: u32,
}

impl DrawResources {
    pub fn new(ctx: &Ctx, shared: &SharedResources, width: u32, height: u32) -> Self {
        let camera = DynamicBuffer::new(
            DynamicBufferDescriptor {
                label: Some("Camera Buffer".into()),
                usage: BufferUsages::UNIFORM,
                kind: DynamicBufferKind::Uniform,
            },
            ctx,
        );

        let bg_layout = DrawBindGroup::layout(ctx);
        let bg = DrawBindGroup::bind_group(ctx, &bg_layout, camera.buffer());

        let shared_bg_layout = SharedBindGroup::layout(ctx);
        let shared_bg = Self::create_shared_bind_group(ctx, &shared_bg_layout, shared);

        Self {
            camera,
            bg,
            bg_layout,
            shared_bg,
            shared_bg_layout,
            width,
            height,
        }
    }

    fn create_shared_bind_group(
        ctx: &Ctx,
        layout: &BindGroupLayout,
        shared: &SharedResources,
    ) -> BindGroup {
        let view = shared
            .out_texture
            .create_view(&TextureViewDescriptor::default());

        SharedBindGroup::bind_group(
            ctx,
            layout,
            (shared.grid.buffer(), shared.map.buffer(), &view),
        )
    }

    pub fn rebuild_shared_bind_group(&mut self, ctx: &Ctx, shared: &SharedResources) {
        self.shared_bg = Self::create_shared_bind_group(ctx, &self.shared_bg_layout, shared);
    }
}

fn init_resources(ctx: Res<Ctx>, shared: Res<SharedResources>, mut commands: Commands) {
    let (width, height) = ctx.size();

    commands.insert_resource(DrawResources::new(&ctx, &shared, width, height));
}

fn init_pipeline(ctx: Res<Ctx>, resources: Res<DrawResources>, mut commands: Commands) {
    commands.insert_resource(DrawPipeline::new(&ctx, &resources));
}

fn extract_camera(
    camera: Query<&Camera, With<Player>>,
    ctx: Res<Ctx>,
    mut resources: ResMut<DrawResources>,
    mut frame: ResMut<Frame>,
) {
    let camera = camera.single().expect("no camera found");

    resources
        .camera
        .write(&[*camera], frame.encoder_mut(), &ctx);
}

fn rebuild(ctx: Res<Ctx>, shared: Res<SharedResources>, mut res: ResMut<DrawResources>) {
    res.rebuild_shared_bind_group(&ctx, &shared);
    info!("draw shared bind group rebuilt");
}

fn dispatch(mut frame: ResMut<Frame>, pipeline: Res<DrawPipeline>, resources: Res<DrawResources>) {
    let encoder = frame.encoder_mut();
    {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Draw Compute Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline.pipeline);
        pass.set_bind_group(0, &resources.shared_bg, &[]);
        pass.set_bind_group(1, &resources.bg, &[]);

        let workgroups_x = resources.width.div_ceil(WORKGROUP_SIZE);
        let workgroups_y = resources.height.div_ceil(WORKGROUP_SIZE);
        pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
    }
}
