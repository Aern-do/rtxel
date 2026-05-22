pub mod command;
use std::sync::Arc;

use bytemuck::NoUninit;
pub use command::*;
use log::info;
use rtxel_gpu::{
    BaseComputePipeline, Ctx, ReadStorageBufer, UniformBuffer, WriteStorageBuffer,
    bind_group::AsBindGroup, group,
};
use wgpu::{
    Buffer, BufferUsages, ComputePassDescriptor, ComputePipeline, PipelineLayoutDescriptor,
};

use crate::render::{Compiler, Frame, GPUMap2, GPUMap4, GPUMap8, GPUWorld};

// using just u32 breaks everything since slang wgsl backend is bugged when using parameter blocks
// for some reason using u32 makes command_size_buffer and command_buffer use same binding
#[derive(Debug, Clone, Copy, PartialEq, Eq, NoUninit)]
#[repr(C)]
struct Size {
    value: u32,
    // uniforms must be aligned to 16 bytes
    _pad: [u32; 3],
}

impl Size {
    fn new(value: u32) -> Self {
        Self {
            value,
            _pad: [0; 3],
        }
    }
}

group!(struct UnpackGroup {
    [compute] command_buffer: ReadStorageBufer<Command>,
    [compute] command_size_buffer: UniformBuffer<Size>,

    [compute] grid_buffer: WriteStorageBuffer<u32>,
    [compute] map8_buffer: WriteStorageBuffer<GPUMap8>,
    [compute] map4_buffer: WriteStorageBuffer<GPUMap4>,
    [compute] map2_buffer: WriteStorageBuffer<GPUMap2>,
});

/// Unpack pass of rendering pipeline
#[derive(Debug)]
pub struct Unpack {
    ctx: Arc<Ctx>,
    pending_commands: Vec<Command>,

    pipeline: ComputePipeline,

    command_buffer: Buffer,
    command_size_buffer: Buffer,
}

impl Unpack {
    /// Maximum amount of unpack commands that can be dispatched in 1 frame
    pub const DISPATCH_SIZE: usize = 2048;
    pub const WORKGROUP_SIZE: usize = 256;

    /// Creates an unpack pass
    pub fn create(ctx: Arc<Ctx>, compiler: &Compiler) -> Self {
        let shader = compiler
            .compile("passes/unpack.slang")
            .expect("failed to compile unpack shader");

        let command_buffer = ctx.create_buffer::<Command>(
            Self::DISPATCH_SIZE,
            Some("Command Buffer"),
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let command_size_buffer = ctx.create_buffer::<Size>(
            1,
            Some("Command Size Buffer"),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Unpack Pipeline Layout"),
                bind_group_layouts: &[Some(&UnpackGroup::layout(&ctx))],
                immediate_size: 0,
            });

        let pipeline = ctx
            .compute_pipeline(BaseComputePipeline {
                layout: &pipeline_layout,
                shader: &shader,
                entry: "main",
            })
            .label("Unpack Pipeline")
            .build();

        Self {
            ctx,
            pending_commands: Vec::new(),
            pipeline,
            command_buffer,
            command_size_buffer,
        }
    }

    /// Queue unpack commands to dispatch
    pub fn queue(&mut self, commands: impl Iterator<Item = Command>) {
        let old = self.pending_commands.len();
        self.pending_commands.extend(commands);
        let amount = self.pending_commands.len() - old;
        if amount > 0 {
            info!("queued {amount} commands to dispatch",)
        }
    }

    pub fn queue_size(&self) -> usize {
        self.pending_commands.len()
    }

    /// Dispatch unpack pass
    pub fn dispatch(&mut self, gpu_world: &GPUWorld, frame: &mut Frame) {
        if self.pending_commands.is_empty() {
            return;
        }

        let dispatch_size = self.pending_commands.len().min(Self::DISPATCH_SIZE);
        info!("dispatching unpack pass with {dispatch_size} command");
        let commands = self
            .pending_commands
            .drain(..dispatch_size)
            .collect::<Vec<_>>();

        let mut pass = frame.encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Unpack Compute Pass"),
            ..Default::default()
        });

        self.ctx
            .queue
            .write_buffer(&self.command_buffer, 0, bytemuck::cast_slice(&commands));
        self.ctx.queue.write_buffer(
            &self.command_size_buffer,
            0,
            bytemuck::bytes_of(&Size::new(dispatch_size as u32)),
        );

        let bind_group = UnpackGroup {
            command_buffer: &self.command_buffer,
            command_size_buffer: &self.command_size_buffer,

            grid_buffer: &gpu_world.grid_buffer,
            map2_buffer: &gpu_world.map2_buffer,
            map4_buffer: &gpu_world.map4_buffer,
            map8_buffer: &gpu_world.map8_buffer,
        }
        .group(&self.ctx);

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(dispatch_size.div_ceil(Self::WORKGROUP_SIZE) as u32, 1, 1);
    }
}
