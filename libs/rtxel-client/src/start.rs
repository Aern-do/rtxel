use log::{info, warn};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Window(WindowEvent),
    Device(DeviceEvent),
}

#[derive(Debug, Clone)]
pub struct Start<N, E, S> {
    new: N,
    event: E,
    attrs: WindowAttributes,

    state: Option<S>,
}

impl<N, E, S> Start<N, E, S> {
    pub fn new(new: N, event: E, attrs: WindowAttributes) -> Self {
        Self {
            new,
            attrs,
            event,
            state: None,
        }
    }

    pub fn run(mut self)
    where
        Self: ApplicationHandler,
    {
        let event_loop = EventLoop::new().expect("failed to create event loop");
        event_loop
            .run_app(&mut self)
            .expect("failed to start application");
    }
}

impl<N, E, S> ApplicationHandler for Start<N, E, S>
where
    N: Fn(Window) -> S,
    E: Fn(&mut S, &ActiveEventLoop, Event),
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            warn!("application has been resumed twice");
            return;
        }

        let window = event_loop
            .create_window(self.attrs.clone())
            .expect("failed to create window");
        let state = (self.new)(window);

        info!("application state created");
        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else {
            warn!("window event received before state is created");
            return;
        };

        let event = Event::Window(event);
        (self.event)(state, event_loop, event)
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let Some(state) = &mut self.state else {
            warn!("window event received before state is created");
            return;
        };

        let event = Event::Device(event);
        (self.event)(state, event_loop, event)
    }
}
