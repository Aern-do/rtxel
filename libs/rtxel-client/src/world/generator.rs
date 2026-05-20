use glam::IVec3;

use crate::world::{Map, Map8, World};

pub fn generate(world: &mut World) {
    let half_x = (world.grid_size().x as i32 * Map8::SIZE as i32) / 2;
    let half_z = (world.grid_size().z as i32 * Map8::SIZE as i32) / 2;
    let half_y = (world.grid_size().y as i32 * Map8::SIZE as i32) / 2;

    for x in -half_x..half_x {
        for z in -half_z..half_z {
            let height = height_at(x, z).min(half_y);
            for y in -half_y..height {
                world.set_voxel(IVec3::new(x, y, z), true);
            }
        }
    }
}

fn height_at(x: i32, z: i32) -> i32 {
    let fx = x as f32 * 0.05;
    let fz = z as f32 * 0.05;
    ((fx.sin() + fz.cos()) * 4.0 + 2.0) as i32
}
