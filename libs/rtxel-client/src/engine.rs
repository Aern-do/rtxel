use std::sync::Arc;

use log::warn;
use rtxel_gpu::Ctx;
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use crate::{
    Event, Start,
    render::{FailedFrame, Frame},
};

#[derive(Debug)]
pub struct Engine {
    pub window: Arc<Window>,
    pub ctx: Arc<Ctx>,
}

impl Engine {
    pub fn new(window: Window) -> Self {
        let window = Arc::new(window);

        let size = window.inner_size();
        let ctx = pollster::block_on(Ctx::new(window.clone(), size.width, size.height));

        Self {
            window,
            ctx: Arc::new(ctx),
        }
    }

    pub fn on_redraw(&mut self) {
        let frame = match Frame::begin(&self.ctx) {
            Ok(frame) => frame,
            Err(FailedFrame::Skip) => return,
            Err(FailedFrame::Outdated) => {
                warn!("surface is outdated, reconfiguration was required");
                self.ctx.reconfigure();
                return;
            }
            Err(FailedFrame::Critical) => panic!("critical error when trying to begin new frame"),
        };

        frame.present(&self.ctx, &self.window);
    }
}

pub fn start() {
    Start::new(
        |window| Engine::new(window),
        handle_event,
        WindowAttributes::default().with_title("Rtxel"),
    )
    .run();
}

fn handle_event(engine: &mut Engine, event_loop: &ActiveEventLoop, event: Event) {
    match event {
        Event::Window(WindowEvent::CloseRequested) => event_loop.exit(),
        Event::Window(WindowEvent::RedrawRequested) => engine.on_redraw(),
        _ => {}
    }
}
