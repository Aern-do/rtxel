use std::sync::Arc;

use rtxel_gpu::{
    AsBindGroup, Ctx, FloatNonFilterable, NonFiltering, Texture2D, TextureSampler, group,
    render_pipeline::BasePipeline,
};
use wgpu::{
    Color, LoadOp, Operations, PipelineLayoutDescriptor, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, Sampler, StoreOp, Texture, TextureViewDescriptor,
    wgt::SamplerDescriptor,
};

use crate::render::{Compiler, Frame};

group!(struct PresentGroup {
    [fragment] output_texture: Texture2D<FloatNonFilterable>,
    [fragment] output_sampler: TextureSampler<NonFiltering>
});

/// Present pass of rendering pipeline.
///
/// Simply takes output texture and presents it onto current surface texture.
#[derive(Debug, Clone)]
pub struct Present {
    pub ctx: Arc<Ctx>,

    pub pipeline: RenderPipeline,
    pub sampler: Sampler,
}

impl Present {
    pub fn new(ctx: Arc<Ctx>, compiler: &Compiler) -> Self {
        let shader = compiler
            .compile("passes/present.slang")
            .expect("failed to compile present shader");
        let sampler = ctx.device.create_sampler(&SamplerDescriptor::default());

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Present Pipeline Layout"),
                bind_group_layouts: &[Some(&PresentGroup::layout(&ctx))],
                immediate_size: 0,
            });

        let pipeline = ctx
            .render_pipeline(BasePipeline {
                layout: &pipeline_layout,
                shader: &shader,
                vertex_entry: "vs_main",
                fragment_entry: "fs_main",
                fragment_format: ctx.config().format,
            })
            .label("Present Pipeline")
            .build();

        Self {
            ctx,
            pipeline,
            sampler,
        }
    }

    pub fn dispatch(&self, frame: &mut Frame, out_texture: &Texture) {
        let view = frame
            .surface
            .texture
            .create_view(&TextureViewDescriptor::default());

        let bind_group = PresentGroup {
            output_texture: &out_texture.create_view(&TextureViewDescriptor::default()),
            output_sampler: &self.sampler,
        }
        .group(&self.ctx);

        let mut pass = frame.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Present Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            ..Default::default()
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
