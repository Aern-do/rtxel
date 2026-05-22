use std::{path::Path, sync::Arc};

pub mod compile;
pub mod debug;
pub mod draw;
pub mod frame;
pub mod gpu_world;
pub mod present;
pub mod slot_allocator;
pub mod unpack;
use bytemuck::{NoUninit, bytes_of};
pub use compile::{Compiler, compile, compile_into_shader_module};
pub use frame::{FailedFrame, Frame};
use glam::{UVec3, Vec3};
pub use gpu_world::{GPUMap2, GPUMap4, GPUMap8, GPUWorld};
pub use present::Present;
use rtxel_gpu::Ctx;
pub use slot_allocator::SlotAllocator;
pub use unpack::{Command, Unpack};
use wgpu::{
    Buffer, BufferUsages, Extent3d, Texture, TextureDimension, TextureFormat, TextureUsages,
    wgt::TextureDescriptor,
};
use winit::window::Window;

use crate::{
    Camera,
    render::{
        debug::{Debug, DebugDispatch, DebugInformation},
        draw::Draw,
    },
    world::{Edit, World},
};

/// Stores additional data needed to render a frame
#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
pub struct RenderData {
    pub grid_size: UVec3,
    pub frame: u32,
}

pub struct Render {
    pub world: GPUWorld,
    pub unpack: Unpack,
    pub present: Present,
    pub draw: Draw,
    pub debug: Debug,

    pub camera_buffer: Buffer,
    pub render_data_buffer: Buffer,
    pub out_texture: Texture,
    pub accum_texture: Texture,
    pub ctx: Arc<Ctx>,

    pub frame: u32,
}

impl Render {
    pub fn new(world: &World, camera: Camera, window: Arc<Window>, ctx: Arc<Ctx>) -> Self {
        let base_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");
        let compiler = Compiler::new(ctx.clone(), base_path);

        let render_data_buffer = ctx.create_buffer_init(
            &[RenderData {
                grid_size: world.grid_size().as_uvec3(),
                frame: 0,
            }],
            Some("Render Data Buffer"),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let camera_buffer = ctx.create_buffer_init(
            &[camera],
            Some("Camera Buffer"),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let size = window.inner_size();

        let out_texture = ctx.device.create_texture(&TextureDescriptor {
            label: Some("Output Texture"),
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let accum_texture = ctx.device.create_texture(&TextureDescriptor {
            label: Some("Accum Texture"),
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        Self {
            world: GPUWorld::new(&ctx, world),
            unpack: Unpack::create(ctx.clone(), &compiler),
            draw: Draw::new(ctx.clone(), &compiler),
            present: Present::new(ctx.clone(), &compiler),
            debug: Debug::new(window, ctx.clone()),
            camera_buffer,
            render_data_buffer,
            out_texture,
            accum_texture,
            ctx,
            frame: 0,
        }
    }

    pub fn update_debug_info(&self, debug_info: &mut DebugInformation) {
        debug_info.dispatch_queue = self.unpack.queue_size();
    }

    /// Update camera buffer
    pub fn update_camera(&self, camera: &Camera) {
        self.ctx
            .queue
            .write_buffer(&self.camera_buffer, 0, bytes_of(camera));
    }

    /// Update render data buffer
    pub fn update_render_data(&mut self, world: &World) {
        self.frame += 1;
        self.ctx.queue.write_buffer(
            &self.render_data_buffer,
            0,
            bytes_of(&RenderData {
                frame: self.frame,
                grid_size: world.grid_size().as_uvec3(),
            }),
        );
    }

    /// Apply world edits
    pub fn apply_edits(&mut self, edits: impl Iterator<Item = Edit>) {
        edits.for_each(|edit| self.world.apply(edit));
    }

    /// Run a rendering pipeline on given frame
    pub fn run(&mut self, frame: &mut Frame, window: &Window, debug_info: DebugInformation) {
        self.unpack.queue(self.world.drain_commands());

        self.unpack.dispatch(&self.world, frame);
        self.draw.dispatch(
            &self.camera_buffer,
            &self.render_data_buffer,
            &self.accum_texture,
            &self.out_texture,
            &self.world,
            window,
            frame,
        );
        self.present.dispatch(frame, &self.out_texture);
        self.debug.render(DebugDispatch {
            frame,
            window,
            debug: debug_info,
        });
    }
}
