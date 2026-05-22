use std::sync::Arc;

use rtxel_gpu::{
    AsBindGroup, BaseComputePipeline, Ctx, Float, NonFiltering, RStorageTexture, RWStorageTexture,
    ReadStorageBufer, Rgba8Unorm, Rgba32Float, Texture2D, TextureSampler, UniformBuffer,
    WStorageTexture, group,
};
use wgpu::{
    Buffer, ComputePassDescriptor, ComputePipeline, PipelineLayoutDescriptor, Sampler, Texture,
    wgt::{SamplerDescriptor, TextureViewDescriptor},
};
use winit::window::Window;

use crate::{
    Camera,
    render::{Compiler, Frame, GPUMap2, GPUMap4, GPUMap8, GPUWorld, RenderData},
};

group!(struct DrawGroup {
    [compute] camera_buffer: UniformBuffer<Camera>,
    [compute] render_data_buffer: UniformBuffer<RenderData>,
    [compute] accum_texture: Texture2D<Float<false>>,
    [compute] out_texture: WStorageTexture<Rgba32Float>,

    [compute] grid_buffer: ReadStorageBufer<u32>,
    [compute] map8_buffer: ReadStorageBufer<GPUMap8>,
    [compute] map4_buffer: ReadStorageBufer<GPUMap4>,
    [compute] map2_buffer: ReadStorageBufer<GPUMap2>,
});

/// Draw pass of rendering pipeline.
///
/// Main pass of entire rendering pipeline, performs path tracing and outputs onto output texture.
#[derive(Debug)]
pub struct Draw {
    pub ctx: Arc<Ctx>,
    pub pipeline: ComputePipeline,
    pub sampler: Sampler,

    pub frame: usize,
}

impl Draw {
    pub fn new(ctx: Arc<Ctx>, compiler: &Compiler) -> Self {
        let shader = compiler
            .compile("passes/draw.slang")
            .expect("failed to compile draw shader");

        let layout = ctx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Draw Pipeline Layout"),
                bind_group_layouts: &[Some(&DrawGroup::layout(&ctx))],
                immediate_size: 0,
            });

        let pipeline = ctx
            .compute_pipeline(BaseComputePipeline {
                layout: &layout,
                shader: &shader,
                entry: "main",
            })
            .label("Draw Compute Pipeline")
            .build();

        let sampler = ctx.device.create_sampler(&SamplerDescriptor::default());

        Self {
            ctx,
            pipeline,
            sampler,
            frame: 0,
        }
    }

    // TODO: wrap all this arguments into 1 structure
    // like DrawDispatchParams or something
    pub fn dispatch(
        &mut self,
        camera_buffer: &Buffer,
        render_data_buffer: &Buffer,
        accum_texture: &Texture,
        out_texture: &Texture,
        world: &GPUWorld,
        window: &Window,
        frame: &mut Frame,
    ) {
        const WORKGROUP_SIZE: u32 = 8;

        let accum_texture = accum_texture.create_view(&TextureViewDescriptor::default());
        let out_texture = out_texture.create_view(&TextureViewDescriptor::default());

        let bind_group = DrawGroup {
            camera_buffer,
            render_data_buffer,
            accum_texture: if self.frame % 2 != 0 {
                &out_texture
            } else {
                &accum_texture
            },
            out_texture: if self.frame % 2 == 0 {
                &out_texture
            } else {
                &accum_texture
            },
            grid_buffer: &world.grid_buffer,
            map8_buffer: &world.map8_buffer,
            map4_buffer: &world.map4_buffer,
            map2_buffer: &world.map2_buffer,
        }
        .group(&self.ctx);

        let size = window.inner_size();

        let mut pass = frame.encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Draw Compute Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);

        let workgroups_x = size.width.div_ceil(WORKGROUP_SIZE);
        let workgroups_y = size.height.div_ceil(WORKGROUP_SIZE);
        pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);

        self.frame += 1;
    }
}
