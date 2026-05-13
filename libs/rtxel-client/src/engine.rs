use std::sync::Arc;

use rtxel_gpu::Ctx;
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use crate::{Event, Start};

#[derive(Debug)]
pub struct Engine {
    pub window: Arc<Window>,
    pub ctx: Ctx,
}

impl Engine {
    pub fn new(window: Window) -> Self {
        let window = Arc::new(window);

        let size = window.inner_size();
        let ctx = pollster::block_on(Ctx::new(window.clone(), size.width, size.height));

        Self { window, ctx }
    }

    pub fn on_redraw(&mut self) {}
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
