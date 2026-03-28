use bevy_ecs::{system::ResMut, world::World};
use glam::ivec3;
use noise::{NoiseFn, Perlin};
use rtxel_core::{Plugin, Startup, WorldExt};

pub mod brick_grid;
pub mod voxel;
pub use brick_grid::*;
pub use voxel::*;

use crate::GpuWorld;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn init(self, world: &mut World) {
        world.insert_resource(BrickGrid::new(32));
        world.add_systems(Startup, generate);
    }
}

fn generate(mut grid: ResMut<BrickGrid>, mut world: ResMut<GpuWorld>) {
    let perlin = Perlin::new(42);

    let brick_size = BrickMap::SIZE as i32;
    let total_size = grid.size as i32 * brick_size;
    let half = total_size / 2;

    let scale = 0.02;
    let max_height = (total_size / 4) as f64;

    for x in -half..half {
        for z in -half..half {
            let height = perlin.get([x as f64 * scale, z as f64 * scale]) * max_height;
            let height = height as i32;

            for y in -half..height.min(half) {
                let edit = grid.set_voxel(ivec3(x, y, z));
                world.apply(edit);
            }
        }
    }
}
