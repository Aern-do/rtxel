use bevy_ecs::world::{Mut, World};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

use crate::{CorePlugin, MouseMotion, Order, Startup, WindowHandle, world_ext::WorldExt};

#[derive(Debug)]
struct Init<F> {
    attach: F,
    attrs: WindowAttributes,
    world: Option<World>,
}

impl<F> Init<F> {
    fn new(attach: F, attrs: WindowAttributes) -> Self {
        Self {
            attach,
            attrs,
            world: None,
        }
    }
}

impl<F: Fn(&mut World, Window)> ApplicationHandler for Init<F> {
    fn resumed(&mut self, active_el: &ActiveEventLoop) {
        if self.world.is_some() {
            return;
        }

        let window = active_el
            .create_window(self.attrs.clone())
            .expect("failed to create window");

        let mut world = World::new();
        (self.attach)(&mut world, window);
        world.run_schedule(Startup);

        self.world = Some(world);
    }

    fn window_event(
        &mut self,
        _active_el: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(world) = &mut self.world else {
            return;
        };

        match event {
            WindowEvent::RedrawRequested => {
                world.resource_scope(|world, order: Mut<Order>| {
                    for &schedule in &order.schedules {
                        world.run_schedule(schedule);
                    }
                });

                world.resource::<WindowHandle>().handle.request_redraw();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let Some(world) = &mut self.world else {
            return;
        };

        match event {
            DeviceEvent::MouseMotion {
                delta: (delta_x, delta_y),
            } => world.trigger(MouseMotion { delta_x, delta_y }),
            _ => {}
        }
    }
}

pub fn start<F: Fn(&mut World)>(attach: F, attrs: WindowAttributes) {
    let el = EventLoop::new().expect("failed to create event loop");

    let mut app = Init::new(
        |world: &mut World, window: Window| {
            world.add_plugin(CorePlugin { window });
            attach(world)
        },
        attrs,
    );

    el.run_app(&mut app).expect("failed to run app")
}
