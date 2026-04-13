use bevy_ecs::{
    query::With,
    resource::Resource,
    schedule::{IntoScheduleConfigs, ScheduleLabel},
    system::{Commands, Query, Res, ResMut},
    world::World,
};
use encase::ShaderType;
use rtxel_core::{Plugin, Startup, WorldExt};
use rtxel_gpu::{
    AsBindGroup, BaseComputePipeline, ComputeBinding, Ctx, DynamicBuffer, DynamicBufferDescriptor,
    DynamicBufferKind, Rgba32Float, UniformBuffer, WStorageTexture,
};
use wgpu::{
    BindGroupLayout, BufferUsages, ComputePassDescriptor, ComputePipeline,
    wgt::TextureViewDescriptor,
};

use crate::{
    BrickGrid, Camera, Frame, PipelineSet, Player, RenderStartupSet,
    shared::{GridBinding, MapBinding, MaterialBinding, PalleteBinding, SharedResources},
};

#[derive(Debug, Default, Clone, Copy, ShaderType)]
pub struct GridConfiguration {
    pub size: i32,
    pub _pad0: i32,
    pub _pad1: i32,
    pub _pad2: i32,
}

type OutputTextureBinding<const IDX: usize> = ComputeBinding<IDX, WStorageTexture<Rgba32Float>>;

type DrawBindGroup = (
    ComputeBinding<0, UniformBuffer<Camera>>,
    ComputeBinding<1, UniformBuffer<GridConfiguration>>,
    OutputTextureBinding<2>,
    GridBinding<3, false>,
    MapBinding<4, false>,
    PalleteBinding<5, false>,
    MaterialBinding<6, false>,
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
                extract_camera.in_set(PipelineSet::Extract),
            )
            .add_systems(self.schedule, dispatch.in_set(PipelineSet::Dispatch));
    }
}

#[derive(Debug, Resource)]
pub struct DrawPipeline {
    pub pipeline: ComputePipeline,
    pub bg_layout: BindGroupLayout,
}

impl DrawPipeline {
    pub fn new(ctx: &Ctx) -> Self {
        let bg_layout = DrawBindGroup::layout(&ctx);
        let layout = ctx.pipeline_layout(Some("Draw Pipeline Layout"), &[Some(&bg_layout)]);

        let pipeline = ctx
            .compute_pipeline(BaseComputePipeline {
                layout: &layout,
                shader: &ctx.wgsl_shader(Some("Draw Shader"), DRAW_SHADER),
                entry: "main",
            })
            .label("Draw Compute Pipeline")
            .build();

        Self {
            pipeline,
            bg_layout,
        }
    }
}

#[derive(Debug, Resource)]
pub struct DrawResources {
    pub camera: DynamicBuffer<Camera>,
    pub configuration: DynamicBuffer<GridConfiguration>,
}

impl DrawResources {
    pub fn new(ctx: &Ctx) -> Self {
        let camera = DynamicBuffer::new(
            DynamicBufferDescriptor {
                label: Some("Camera Buffer".into()),
                usage: BufferUsages::UNIFORM,
                kind: DynamicBufferKind::Uniform,
            },
            ctx,
        );

        let configuration = DynamicBuffer::new(
            DynamicBufferDescriptor {
                label: Some("Configuration Buffer".into()),
                usage: BufferUsages::UNIFORM,
                kind: DynamicBufferKind::Uniform,
            },
            ctx,
        );

        Self {
            camera,
            configuration,
        }
    }
}

fn init_resources(ctx: Res<Ctx>, mut commands: Commands) {
    commands.insert_resource(DrawResources::new(&ctx));
}

fn init_pipeline(ctx: Res<Ctx>, mut commands: Commands) {
    commands.insert_resource(DrawPipeline::new(&ctx));
}

fn extract_camera(
    camera: Query<&Camera, With<Player>>,
    ctx: Res<Ctx>,
    grid: Res<BrickGrid>,
    mut resources: ResMut<DrawResources>,
    mut frame: ResMut<Frame>,
) {
    let camera = camera.single().expect("no camera found");

    resources
        .camera
        .write(&[*camera], frame.encoder_mut(), &ctx);

    resources.configuration.write(
        &[GridConfiguration {
            size: grid.size as i32,
            ..Default::default()
        }],
        frame.encoder_mut(),
        &ctx,
    );
}

fn dispatch(
    mut frame: ResMut<Frame>,
    pipeline: Res<DrawPipeline>,
    resources: Res<DrawResources>,
    shared_res: Res<SharedResources>,
    ctx: Res<Ctx>,
) {
    let (width, height) = ctx.size();
    let bg = DrawBindGroup::bind_group(
        &ctx,
        &pipeline.bg_layout,
        (
            resources.camera.buffer(),
            resources.configuration.buffer(),
            &shared_res
                .out_texture
                .create_view(&TextureViewDescriptor::default()),
            shared_res.grid.buffer(),
            shared_res.map.buffer(),
            shared_res.palletes.buffer(),
            shared_res.materials.buffer(),
        ),
    );

    let encoder = frame.encoder_mut();
    {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Draw Compute Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline.pipeline);
        pass.set_bind_group(0, &bg, &[]);

        let workgroups_x = width.div_ceil(WORKGROUP_SIZE);
        let workgroups_y = height.div_ceil(WORKGROUP_SIZE);
        pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
    }
}
