use std::{sync::Arc, time::Instant};

use glam::{USizeVec3, Vec3};
use log::{info, warn};
use rtxel_gpu::Ctx;
use winit::{
    event::{DeviceEvent, ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowAttributes},
};

use crate::{
    Camera, Event, Keyboard, Start,
    render::{FailedFrame, Frame, Render, debug::DebugInformation},
    world::{World, generator::generate},
};

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
    pub camera_preset: usize,
    pub enable: bool,
}

impl Engine {
    const CAMERA_PRESETS: &[(Vec3, f32, f32)] = &[
        (Vec3::new(123.0, -18.0, 403.0), -91.2, -13.0),
        (Vec3::new(-128.0, -100.0, -195.0), -264.8, -26.4),
        (Vec3::new(124.0, 239.0, -858.0), -264.8, -26.4),
    ];

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
        let render = Render::new(&world, camera, window.clone(), ctx.clone());

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
            camera_preset: 0,
            enable: false,
        }
    }

    pub fn on_redraw(&mut self) {
        let mut frame = match self.begin_frame() {
            Some(frame) => frame,
            None => return,
        };

        let mut debug_info = DebugInformation::default();

        self.tick(&mut debug_info);
        self.render.run(&mut frame, &self.window, debug_info);

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

    fn tick(&mut self, debug_info: &mut DebugInformation) {
        self.window
            .set_cursor_grab(CursorGrabMode::Locked)
            .expect("failed to lock mouse");
        let now = Instant::now();
        self.dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        debug_info.dt = self.dt;

        self.camera.frame += 1;
        self.camera.update_keyboard(&self.keyboard, self.dt);

        self.render.apply_edits(self.world.drain_edits());
        self.render.update_debug_info(debug_info);
        self.render.update_camera(&self.camera);
        self.render.update_render_data(&self.world, self.enable);

        if self.keyboard.just_pressed(KeyCode::KeyP) {
            let (origin, yaw, pitch) = Self::CAMERA_PRESETS[self.camera_preset];
            self.camera.origin = origin;
            self.camera.yaw = yaw;
            self.camera.pitch = pitch;
            self.camera.update_vectors();
            self.camera.frame = 0;
            self.camera_preset = (self.camera_preset + 1) % Self::CAMERA_PRESETS.len();
        }

        if self.keyboard.just_pressed(KeyCode::KeyO) {
            info!("enabled debug");
            self.enable = !self.enable;
        }

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
