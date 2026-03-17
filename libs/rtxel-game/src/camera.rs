use bevy_ecs::{
    component::Component,
    observer::On,
    query::With,
    system::{Commands, Query, Res},
    world::World,
};
use glam::Vec3;
use rtxel_core::{MouseMotion, Plugin, Startup, Update, WindowHandle, WorldExt};

use crate::Player;

#[derive(Debug, Clone, Copy, Component)]
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
}

const SENSITIVITY: f32 = 0.1;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn init(self, world: &mut World) {
        world.add_systems(Startup, setup_camera);
        world.add_observer(mouse_look);
    }
}

fn setup_camera(window: Res<WindowHandle>, mut commands: Commands) {
    let size = window.handle.inner_size();

    let mut camera = Camera {
        origin: Vec3::new(0., 1., -1.),
        forward: Vec3::Z,
        up: Vec3::Y,
        aspect: size.width as f32 / size.height as f32,
        fov: 65.0,
        yaw: 90.0,
        pitch: 0.0,
    };
    camera.update_vectors();

    commands.spawn((Player, camera));
}

fn mouse_look(motion: On<MouseMotion>, mut camera: Query<&mut Camera, With<Player>>) {
    let mut total_x = 0.0;
    let mut total_y = 0.0;

    total_x += motion.delta_x as f32;
    total_y += motion.delta_y as f32;

    if total_x == 0.0 && total_y == 0.0 {
        return;
    }

    let mut cam = camera.single_mut().expect("expected camera");

    cam.yaw += total_x * SENSITIVITY;
    cam.pitch -= total_y * SENSITIVITY;
    cam.pitch = cam.pitch.clamp(-89.0, 89.0);

    cam.update_vectors();
}
