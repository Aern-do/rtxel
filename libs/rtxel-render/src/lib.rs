pub mod base;

use bevy_ecs::{
    resource::Resource,
    schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel},
    system::{Commands, Res, ResMut},
    world::World,
};
use pollster::block_on;
use rtxel_core::{Order, Plugin, PostUpdate, Startup, WindowHandle, WorldExt};
use rtxel_gpu::Ctx;
use wgpu::{CommandEncoder, CommandEncoderDescriptor, SurfaceTexture};

#[derive(Debug, Default, Resource)]
pub struct Frame {
    pub encoder: Option<CommandEncoder>,
    pub tex: Option<SurfaceTexture>,
}

impl Frame {
    pub fn encoder(&mut self) -> &mut CommandEncoder {
        self.encoder.as_mut().expect("command encoder is missing")
    }

    pub fn tex(&mut self) -> &mut SurfaceTexture {
        self.tex.as_mut().expect("surface texture is missing")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Extract;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Render;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn init(self, world: &mut World) {
        world.add_schedule(Schedule::new(Extract));
        world.add_schedule(Schedule::new(Render));
        let mut order = world.resource_mut::<Order>();

        order.insert_after(PostUpdate, Extract);
        order.insert_after(Extract, Render);

        world.init_resource::<Frame>();

        world.add_systems(Startup, (init_ctx, base::init).chain());
        world.add_systems(Extract, base::extract);
        world.add_systems(Render, (begin_frame, base::render, end_frame).chain());
    }
}

pub fn init_ctx(mut commands: Commands, window: Res<WindowHandle>) {
    let size = window.handle.inner_size();

    let ctx = block_on(Ctx::new(window.handle.clone(), size.width, size.height));
    commands.insert_resource(ctx);
}

pub fn begin_frame(ctx: Res<Ctx>, mut frame: ResMut<Frame>) {
    let encoder = ctx
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });

    let tex = ctx
        .surface
        .get_current_texture()
        .expect("failed to get surface texture");

    frame.encoder = Some(encoder);
    frame.tex = Some(tex)
}

pub fn end_frame(ctx: Res<Ctx>, mut frame: ResMut<Frame>) {
    let encoder = frame.encoder.take().expect("command encoder is missing");
    let tex = frame.tex.take().expect("surface texture is missing");

    ctx.queue.submit(Some(encoder.finish()));
    tex.present();
}
