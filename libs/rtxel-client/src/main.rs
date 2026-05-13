pub mod start;
use std::sync::Arc;

pub use start::{Event, Start};
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

pub struct State {
    pub window: Arc<Window>,
}

fn main() {
    Start::new(
        |window| State {
            window: Arc::new(window),
        },
        handle_event,
        WindowAttributes::default(),
    )
    .run();
}

fn handle_event(_state: &mut State, event_loop: &ActiveEventLoop, event: Event) {
    match event {
        Event::Window(WindowEvent::CloseRequested) => event_loop.exit(),
        _ => {}
    }
}
