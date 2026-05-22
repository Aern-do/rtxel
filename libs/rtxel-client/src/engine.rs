use std::{sync::Arc, time::Instant};

use glam::USizeVec3;
use log::warn;
use rtxel_gpu::Ctx;
use winit::{
    event::{DeviceEvent, ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowAttributes},
};

use crate::{
    Camera, Event, Keyboard, Start,
    render::{FailedFrame, Frame, Render},
    world::{World, generator::generate},
};

#[derive(Debug)]
pub struct Engine {
    pub window: Arc<Window>,
    pub ctx: Arc<Ctx>,
    pub keyboard: Keyboard,
    pub camera: Camera,

    pub world: World,
    pub render: Render,

    pub skipped_frames: usize,
    pub last_frame: Instant,
    pub dt: f32,
}

impl Engine {
    pub fn new(window: Window) -> Self {
        let window = Arc::new(window);
        let size = window.inner_size();
        let ctx = Arc::new(pollster::block_on(Ctx::new(
            window.clone(),
            size.width,
            size.height,
        )));

        let mut world = World::new(USizeVec3::new(128, 128, 128));

        let camera = Camera::new(size.width as f32 / size.height as f32);
        let render = Render::new(&world, camera, &window, ctx.clone());

        generate(&mut world);
        world.emit_all_edits();

        Self {
            window,
            ctx,
            keyboard: Keyboard::new(),
            camera,
            world,
            render,
            skipped_frames: 0,
            last_frame: Instant::now(),
            dt: 0.0,
        }
    }

    pub fn on_redraw(&mut self) {
        let mut frame = match self.begin_frame() {
            Some(frame) => frame,
            None => return,
        };

        self.tick();
        self.render.run(&mut frame, &self.window);

        self.window.pre_present_notify();
        frame.present(&self.ctx);
        self.window.request_redraw();

        self.report_skipped();
    }

    fn begin_frame(&mut self) -> Option<Frame> {
        match Frame::begin(&self.ctx) {
            Ok(frame) => Some(frame),
            Err(FailedFrame::Skip) => {
                self.skipped_frames += 1;
                self.window.request_redraw();
                None
            }
            Err(FailedFrame::Outdated) => {
                warn!("surface outdated, reconfiguring");
                self.ctx.reconfigure();
                self.window.request_redraw();
                None
            }
            Err(FailedFrame::Critical) => panic!("critical error beginning frame"),
        }
    }

    fn tick(&mut self) {
        self.window
            .set_cursor_grab(CursorGrabMode::Locked)
            .expect("failed to lock mouse");
        let now = Instant::now();
        self.dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        self.camera.frame += 1;
        self.camera.update_keyboard(&self.keyboard, self.dt);

        self.render.apply_edits(self.world.drain_edits());
        self.render.update_camera(&self.camera);
        self.render.update_render_data(&self.world);
        self.keyboard.clear();
    }

    fn report_skipped(&mut self) {
        if self.skipped_frames > 0 {
            warn!("skipped {} frames!", self.skipped_frames);
            self.skipped_frames = 0;
        }
    }

    pub fn on_keyboard_input(&mut self, key: KeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => self.keyboard.press(key),
            ElementState::Released => self.keyboard.release(key),
        }
    }

    pub fn on_mouse_input(&mut self, dx: f64, dy: f64) {
        self.camera.update_mouse(dx as f32, dy as f32);
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
        Event::Window(WindowEvent::KeyboardInput { event, .. })
            if let PhysicalKey::Code(key) = event.physical_key =>
        {
            engine.on_keyboard_input(key, event.state)
        }
        Event::Device(DeviceEvent::MouseMotion { delta }) => {
            engine.on_mouse_input(delta.0, delta.1)
        }
        _ => {}
    }
}
