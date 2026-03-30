use bevy_ecs::{system::ResMut, world::World};
use glam::ivec3;
use rand::{RngExt, SeedableRng, rngs::SmallRng};
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
    let mut rng = SmallRng::seed_from_u64(128);

    let brick_size = BrickMap::SIZE as i32;
    let total_size = grid.size as i32 * brick_size;
    let half = total_size / 2;

    for x in -half..half {
        for y in -half..half {
            for z in -half..half {
                if rng.random_bool(0.01) {
                    let edit = grid.set_voxel(ivec3(x, y, z));
                    world.apply(edit);
                }
            }
        }
    }
}
