use bevy_ecs::{
    resource::Resource,
    schedule::{IntoScheduleConfigs, ScheduleLabel, SystemSet},
    system::{Commands, Res, ResMut},
    world::World,
};
use pollster::block_on;
use rtxel_core::{Order, Plugin, PostUpdate, Startup, WindowHandle, WorldExt};
use rtxel_gpu::Ctx;
use wgpu::{CommandEncoder, CommandEncoderDescriptor, SurfaceTexture};

pub mod gpu_world;
pub mod pipeline;
pub mod resources;
pub mod slot_allocator;
pub use gpu_world::*;
pub use pipeline::*;
pub use resources::*;
pub use slot_allocator::*;

#[derive(Debug, Default, Resource)]
pub struct Frame {
    pub surface: Option<SurfaceTexture>,
    pub encoder: Option<CommandEncoder>,
}

impl Frame {
    pub fn surface(&self) -> &SurfaceTexture {
        self.surface
            .as_ref()
            .expect("surface texture is missing from frame")
    }

    pub fn encoder_mut(&mut self) -> &mut CommandEncoder {
        self.encoder
            .as_mut()
            .expect("command encoder is missing from frame")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct BeginFrame;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct EndFrame;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Clean;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum RenderStartupSet {
    Context,
    SharedResources,
    Resources,
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn init(self, world: &mut World) {
        world.insert_resource(GpuWorld::new(16));
        world.init_resource::<Frame>();

        let mut order = world.resource_mut::<Order>();
        order.insert_many_after(
            PostUpdate,
            &[BeginFrame.intern(), EndFrame.intern(), Clean.intern()],
        );

        world.configure_sets(
            Startup,
            (
                RenderStartupSet::Context,
                RenderStartupSet::SharedResources,
                RenderStartupSet::Resources,
            )
                .chain(),
        );

        world.add_systems(
            Startup,
            (
                init_ctx.in_set(RenderStartupSet::Context),
                init_resources.in_set(RenderStartupSet::Resources),
            ),
        );
        world
            .add_systems(BeginFrame, begin_frame)
            .add_systems(EndFrame, end_frame);

        world.add_plugin(PipelinePlugin {
            begin_frame: BeginFrame,
            clean: Clean,
        });
    }
}

fn init_ctx(mut commands: Commands, window: Res<WindowHandle>) {
    let size = window.handle.inner_size();

    let ctx = block_on(Ctx::new(window.handle.clone(), size.width, size.height));
    commands.insert_resource(ctx);
}

fn init_resources(ctx: Res<Ctx>, mut commands: Commands) {
    let (width, height) = ctx.size();
    commands.insert_resource(Resources::new(width, height, &ctx));
}

fn begin_frame(ctx: Res<Ctx>, mut frame: ResMut<Frame>) {
    let encoder = ctx
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });

    let surface = ctx
        .surface
        .get_current_texture()
        .expect("failed to get surface texture");

    frame.encoder = Some(encoder);
    frame.surface = Some(surface)
}

fn end_frame(ctx: Res<Ctx>, mut frame: ResMut<Frame>) {
    let encoder = frame.encoder.take().expect("command encoder is missing");
    let tex = frame.surface.take().expect("surface texture is missing");

    ctx.queue.submit(Some(encoder.finish()));
    tex.present();
}
