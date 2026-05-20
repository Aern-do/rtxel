use std::{path::Path, sync::Arc};

pub mod compile;
pub mod frame;
pub mod gpu_world;
pub mod slot_allocator;
pub mod unpack;

use bytemuck::bytes_of;
pub use compile::{Compiler, compile, compile_into_shader_module};
pub use frame::{FailedFrame, Frame};
pub use gpu_world::{GPUMap2, GPUMap4, GPUMap8, GPUWorld};
use rtxel_gpu::Ctx;
pub use slot_allocator::SlotAllocator;
pub use unpack::{Command, Unpack};
use wgpu::{Buffer, BufferUsages};

use crate::{
    Camera,
    world::{Edit, World},
};

#[derive(Debug)]
pub struct Render {
    pub world: GPUWorld,
    pub unpack: Unpack,

    pub camera_buffer: Buffer,
    pub ctx: Arc<Ctx>,
}

impl Render {
    pub fn new(world: &World, camera: Camera, ctx: Arc<Ctx>) -> Self {
        let base_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");
        let compiler = Compiler::new(ctx.clone(), base_path);

        let camera_buffer = ctx.create_buffer_init(
            &[camera],
            Some("Camera Buffer"),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        Self {
            world: GPUWorld::new(&ctx, world),
            unpack: Unpack::create(ctx.clone(), &compiler),
            camera_buffer,
            ctx,
        }
    }

    /// Update camera buffer
    pub fn update_camera(&mut self, camera: &Camera) {
        self.ctx
            .queue
            .write_buffer(&self.camera_buffer, 0, bytes_of(camera));
    }

    /// Apply world edits
    pub fn apply_edits(&mut self, edits: impl Iterator<Item = Edit>) {
        edits.for_each(|edit| self.world.apply(edit));
    }

    /// Run a rendering pipeline on given frame
    pub fn run(&mut self, frame: &mut Frame) {
        self.unpack.queue(self.world.drain_commands());
        self.unpack.dispatch(&self.world, frame);
    }
}
