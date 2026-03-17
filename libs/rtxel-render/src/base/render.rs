use bevy_ecs::system::{Res, ResMut};
use rtxel_gpu::Ctx;
use wgpu::{
    Color, ComputePassDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureViewDescriptor,
};

use crate::{Frame, base::Pipeline};

pub fn render(mut frame: ResMut<Frame>, ctx: Res<Ctx>, pipeline: Res<Pipeline>) {
    let config = ctx.config.lock().unwrap();
    let width = config.width;
    let height = config.height;

    drop(config);

    let view = frame
        .tex()
        .texture
        .create_view(&TextureViewDescriptor::default());

    {
        let mut pass = frame.encoder().begin_compute_pass(&ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline.compute.pipeline);
        pass.set_bind_group(0, &pipeline.compute.bg, &[]);
        pass.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }

    {
        let mut pass = frame.encoder().begin_render_pass(&RenderPassDescriptor {
            label: Some("blit_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
        pass.set_pipeline(&pipeline.draw.pipeline);
        pass.set_bind_group(0, &pipeline.draw.bg, &[]);
        pass.draw(0..3, 0..1);
    }
}
