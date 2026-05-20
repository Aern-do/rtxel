use std::{path::Path, sync::Arc};

pub mod compile;
pub mod frame;
pub mod gpu_world;
pub mod slot_allocator;
pub mod unpack;

pub use compile::{Compiler, compile, compile_into_shader_module};
pub use frame::{FailedFrame, Frame};
pub use gpu_world::{GPUMap2, GPUMap4, GPUMap8, GPUWorld};
use rtxel_gpu::Ctx;
pub use slot_allocator::SlotAllocator;
pub use unpack::{Command, Unpack};

use crate::world::{Edit, World};

#[derive(Debug)]
pub struct Render {
    pub world: GPUWorld,
    pub unpack: Unpack,
}

impl Render {
    pub fn new(world: &World, ctx: Arc<Ctx>) -> Self {
        let base_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");
        let compiler = Compiler::new(ctx.clone(), base_path);

        Self {
            world: GPUWorld::new(&ctx, world),
            unpack: Unpack::create(ctx, &compiler),
        }
    }

    /// Apply world edits
    pub fn apply_edits(&mut self, edits: impl Iterator<Item = Edit>) {
        edits.for_each(|edit| self.world.apply(edit));
    }

    /// Render a Frame
    pub fn render(&mut self, frame: &mut Frame) {
        self.unpack.queue(self.world.drain_commands());
        self.unpack.dispatch(&self.world, frame);
    }
}
