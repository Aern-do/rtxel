use std::sync::Arc;

use egui::{Context, Grid, ViewportId, Window as EguiWindow, pos2};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use egui_winit::{EventResponse, State};
use rtxel_gpu::Ctx;
use wgpu::{
    LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp,
    wgt::TextureViewDescriptor,
};
use winit::{event::WindowEvent, window::Window};

use crate::render::Frame;

/// Struct holding all the debug information, gets tossed around engine to collect everything needed
#[derive(Debug, Default, Clone, Copy)]
pub struct DebugInformation {
    pub dt: f32,
    pub dispatch_queue: usize,
}

/// Everything required to dispatch a debug pass
pub struct DebugDispatch<'disp> {
    pub frame: &'disp mut Frame,
    pub window: &'disp Window,
    pub debug: DebugInformation,
}

pub struct Debug {
    pub ctx: Arc<Ctx>,
    pub window: Arc<Window>,

    pub egui_ctx: Context,
    pub state: State,
    pub renderer: Renderer,
}

impl Debug {
    pub fn new(window: Arc<Window>, ctx: Arc<Ctx>) -> Self {
        let egui_ctx = Context::default();

        let state = State::new(
            egui_ctx.clone(),
            ViewportId::ROOT,
            &window,
            None,
            None,
            None,
        );

        let surface_format = ctx.config().format;
        let renderer = Renderer::new(&ctx.device, surface_format, RendererOptions::default());

        Self {
            ctx,
            window,

            egui_ctx,
            state,
            renderer,
        }
    }

    pub fn on_window_event(&mut self, window: &Window, event: &WindowEvent) -> EventResponse {
        self.state.on_window_event(window, event)
    }

    pub fn render(&mut self, dispatch: DebugDispatch) {
        let raw_input = self.state.take_egui_input(&self.window);
        let full_output = self.egui_ctx.run_ui(raw_input, |ctx| {
            Self::debug_ui(ctx, dispatch.debug);
        });

        self.state
            .handle_platform_output(&self.window, full_output.platform_output);

        let pixels_per_point = self.window.scale_factor() as f32;
        let paint_jobs = self
            .egui_ctx
            .tessellate(full_output.shapes, pixels_per_point);

        let size = self.window.inner_size();
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point,
        };

        for (texture_id, image_delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(
                &self.ctx.device,
                &self.ctx.queue,
                *texture_id,
                image_delta,
            );
        }

        self.renderer.update_buffers(
            &self.ctx.device,
            &self.ctx.queue,
            &mut dispatch.frame.encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        let view = dispatch
            .frame
            .surface
            .texture
            .create_view(&TextureViewDescriptor::default());

        {
            let render_pass = dispatch
                .frame
                .encoder
                .begin_render_pass(&RenderPassDescriptor {
                    label: Some("Debug Overlay Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });

            self.renderer.render(
                &mut render_pass.forget_lifetime(),
                &paint_jobs,
                &screen_descriptor,
            );
        }

        for texture_id in &full_output.textures_delta.free {
            self.renderer.free_texture(texture_id);
        }
    }

    fn debug_ui(ctx: &Context, debug: DebugInformation) {
        let frame_time_ms = debug.dt * 1000.0;

        let fps = if debug.dt > f32::EPSILON {
            1.0 / debug.dt
        } else {
            0.0
        };

        EguiWindow::new("Debug")
            .default_pos(pos2(12.0, 12.0))
            .resizable(false)
            .collapsible(true)
            .show(ctx, |ui| {
                Grid::new("debug")
                    .num_columns(2)
                    .spacing([16.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("FPS");
                        ui.monospace(format!("{fps:.1}"));
                        ui.end_row();

                        ui.label("Frame time");
                        ui.monospace(format!("{frame_time_ms:.3} ms"));
                        ui.end_row();

                        ui.label("Dispatch queue");
                        ui.monospace(debug.dispatch_queue.to_string());
                        ui.end_row();
                    });
            });
    }
}
