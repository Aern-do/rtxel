use std::sync::Arc;

use glam::USizeVec3;
use log::warn;
use rtxel_gpu::Ctx;
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use crate::{
    Event, Start,
    render::{FailedFrame, Frame, Render},
    world::{World, generator::generate},
};

#[derive(Debug)]
pub struct Engine {
    pub window: Arc<Window>,
    pub ctx: Arc<Ctx>,

    pub world: World,
    pub render: Render,

    pub skipped_frames: usize,
}

impl Engine {
    pub fn new(window: Window) -> Self {
        let window = Arc::new(window);

        let size = window.inner_size();
        let ctx = pollster::block_on(Ctx::new(window.clone(), size.width, size.height));
        let ctx = Arc::new(ctx);

        let mut world = World::new(USizeVec3::new(8, 8, 8));
        generate(&mut world);
        let render = Render::new(&world, ctx.clone());

        Self {
            ctx: ctx.clone(),
            window,
            render,
            world,
            skipped_frames: 0,
        }
    }

    pub fn on_redraw(&mut self) {
        let mut frame = match Frame::begin(&self.ctx) {
            Ok(frame) => frame,
            Err(FailedFrame::Skip) => {
                self.skipped_frames += 1;
                self.window.request_redraw();
                return;
            }
            Err(FailedFrame::Outdated) => {
                warn!("surface is outdated, reconfiguration was required");
                self.ctx.reconfigure();
                self.window.request_redraw();
                return;
            }
            Err(FailedFrame::Critical) => panic!("critical error when trying to begin new frame"),
        };

        self.render.apply_edits(self.world.drain_edits());
        self.render.render(&mut frame);

        self.window.pre_present_notify();
        frame.present(&self.ctx);
        self.window.request_redraw();

        if self.skipped_frames > 0 {
            warn!("skipped {} frames!", self.skipped_frames)
        }
        self.skipped_frames = 0;
    }
}

pub fn start() {
    Start::new(
        Engine::new,
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
