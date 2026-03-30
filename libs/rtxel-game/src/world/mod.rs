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
    let seed = 128;
    let perlin = Perlin::new(seed);

    let brick_size = BrickMap::SIZE as i32;
    let total_size = grid.size as i32 * brick_size;
    let half = total_size / 2;

    let frequency = 0.05;
    let amplitude = (total_size as f64) * 0.1;

    let detail_freq = 0.08;
    let detail_amp = amplitude * 0.;

    for x in -half..half {
        for z in -half..half {
            let nx = x as f64 * frequency;
            let nz = z as f64 * frequency;

            let height = perlin.get([nx, nz]) * amplitude
                + perlin.get([x as f64 * detail_freq, z as f64 * detail_freq]) * detail_amp;

            let max_y = height.round() as i32;

            for y in -half..max_y.min(half) {
                let edit = grid.set_voxel(ivec3(x, y, z));
                world.apply(edit);
            }
        }
    }
}
