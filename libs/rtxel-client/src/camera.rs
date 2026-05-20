use bytemuck::NoUninit;
use glam::Vec3;
use winit::keyboard::KeyCode;

use crate::Keyboard;

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
pub struct Camera {
    pub origin: Vec3,
    pub yaw: f32,
    pub pitch: f32,

    pub forward: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fov: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            origin: Vec3::new(0., 64., -1.),
            yaw: 65.0,
            pitch: 90.0,
            forward: Vec3::Z,
            up: Vec3::Y,
            aspect,
            fov: 90.0,
        }
    }

    pub fn update_mouse(&mut self, dx: f32, dy: f32) {
        // TODO: settings
        const SENSITIVITY: f32 = 0.1;

        self.yaw += dx * SENSITIVITY;
        self.pitch -= dy * SENSITIVITY;
        self.pitch = self.pitch.clamp(-89.9, 89.9);

        self.update_vectors();
    }

    pub fn update_keyboard(&mut self, keyboard: &Keyboard, dt: f32) {
        const SENSITIVITY: f32 = 45.0;
        let (forward, right, _) = self.vectors();

        if keyboard.is_pressed(KeyCode::KeyW) {
            self.origin += forward * dt * SENSITIVITY;
        }
        if keyboard.is_pressed(KeyCode::KeyS) {
            self.origin -= forward * dt * SENSITIVITY;
        }
        if keyboard.is_pressed(KeyCode::KeyD) {
            self.origin += right * dt * SENSITIVITY;
        }
        if keyboard.is_pressed(KeyCode::KeyA) {
            self.origin -= right * dt * SENSITIVITY;
        }
    }

    pub fn update_vectors(&mut self) {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        self.forward = Vec3::new(
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.sin() * pitch_rad.cos(),
        )
        .normalize();

        let world_up = Vec3::Y;
        let right = world_up.cross(self.forward).normalize();
        self.up = self.forward.cross(right).normalize();
    }

    pub fn vectors(&self) -> (Vec3, Vec3, Vec3) {
        let right = Vec3::Y.cross(self.forward).normalize();
        let up = self.forward.cross(right).normalize();
        (self.forward, right, up)
    }
}
