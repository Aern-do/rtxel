use bevy_ecs::{
    query::With,
    system::{Query, Res},
};
use encase::UniformBuffer;
use glam::Vec3;
use rtxel_game::{Camera, Player};
use rtxel_gpu::Ctx;

use crate::base::{CameraUniform, Pipeline};

pub fn extract(ctx: Res<Ctx>, pipeline: Res<Pipeline>, camera: Query<&Camera, With<Player>>) {
    let cam = camera.single().expect("expected camera");

    let yaw_rad = cam.yaw.to_radians();
    let pitch_rad = cam.pitch.to_radians();

    let forward = Vec3::new(
        yaw_rad.cos() * pitch_rad.cos(),
        pitch_rad.sin(),
        yaw_rad.sin() * pitch_rad.cos(),
    )
    .normalize();

    let world_up = Vec3::Y;
    let right = forward.cross(world_up).normalize();
    let up = right.cross(forward).normalize();

    let uniform = CameraUniform {
        origin: cam.origin,
        forward,
        up,
        right,
        fov: cam.fov,
        aspect: cam.aspect,
    };

    let mut buffer = UniformBuffer::new(Vec::new());
    buffer.write(&uniform).unwrap();

    ctx.queue
        .write_buffer(&pipeline.compute.camera_buffer, 0, buffer.as_ref());
}
