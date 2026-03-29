use bevy_ecs::{
    component::Component,
    observer::On,
    query::With,
    system::{Commands, Query, Res},
    world::World,
};
use encase::ShaderType;
use glam::Vec3;
use rtxel_core::{
    DeltaTime, KeyCode, Keyboard, MouseMotion, Plugin, Startup, Update, WindowHandle, WorldExt,
};
use winit::window::CursorGrabMode;

use crate::Player;

#[derive(Debug, Clone, Copy, ShaderType, Component)]
pub struct Camera {
    pub origin: Vec3,
    pub yaw: f32,
    pub pitch: f32,

    pub forward: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fov: f32,

    pub frame_count: u32,
}

impl Camera {
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
        let right = self.forward.cross(world_up).normalize();
        self.up = right.cross(self.forward).normalize();
    }

    pub fn vectors(&self) -> (Vec3, Vec3, Vec3) {
        let right = self.forward.cross(Vec3::Y).normalize();
        let up = right.cross(self.forward).normalize();
        (self.forward, right, up)
    }
}

const SENSITIVITY: f32 = 0.1;
const MOVE_SENSITIVITY: f32 = 15.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn init(self, world: &mut World) {
        world.add_systems(Startup, setup_camera);
        world.add_systems(Update, (increment_camera, move_camera));
        world.add_observer(mouse_look);
    }
}

fn setup_camera(window: Res<WindowHandle>, mut commands: Commands) {
    let size = window.handle.inner_size();
    window
        .handle
        .set_cursor_grab(CursorGrabMode::Locked)
        .expect("failed to grab cursor");
    window.handle.set_cursor_visible(false);

    let mut camera = Camera {
        origin: Vec3::new(0., 1., -1.),
        forward: Vec3::Z,
        up: Vec3::Y,
        aspect: size.width as f32 / size.height as f32,
        fov: 65.0,
        yaw: 90.0,
        pitch: 0.0,
        frame_count: 0,
    };
    camera.update_vectors();

    commands.spawn((Player, camera));
}

fn mouse_look(motion: On<MouseMotion>, mut camera: Query<&mut Camera, With<Player>>) {
    if motion.delta_x == 0.0 && motion.delta_y == 0.0 {
        return;
    }

    let mut cam = camera.single_mut().expect("expected camera");

    cam.yaw += motion.delta_x as f32 * SENSITIVITY;
    cam.pitch -= motion.delta_y as f32 * SENSITIVITY;
    cam.pitch = cam.pitch.clamp(-89.0, 89.0);

    cam.update_vectors();
}

fn increment_camera(mut camera: Query<&mut Camera, With<Player>>) {
    let mut camera = camera.single_mut().expect("expected camera");
    camera.frame_count += 1;
}

fn move_camera(
    keyboard: Res<Keyboard>,
    delta: Res<DeltaTime>,
    mut camera: Query<&mut Camera, With<Player>>,
) {
    let mut camera = camera.single_mut().expect("expected camera");
    let (forward, right, _) = camera.vectors();

    if keyboard.is_pressed(KeyCode::KeyW) {
        camera.origin += forward * delta.seconds * MOVE_SENSITIVITY;
    }
    if keyboard.is_pressed(KeyCode::KeyS) {
        camera.origin -= forward * delta.seconds * MOVE_SENSITIVITY;
    }
    if keyboard.is_pressed(KeyCode::KeyD) {
        camera.origin += right * delta.seconds * MOVE_SENSITIVITY;
    }
    if keyboard.is_pressed(KeyCode::KeyA) {
        camera.origin -= right * delta.seconds * MOVE_SENSITIVITY;
    }
}
