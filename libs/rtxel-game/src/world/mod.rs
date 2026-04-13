use bevy_ecs::{
    resource::Resource,
    schedule::IntoScheduleConfigs,
    system::{Commands, Res, ResMut},
    world::World,
};
use glam::{Vec3, ivec3};
use log::info;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use rtxel_core::{Plugin, Startup, WorldExt};

pub mod brick_grid;
pub mod material;
pub mod voxel;
pub use brick_grid::*;
pub use material::*;
pub use voxel::*;

use crate::GpuWorld;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn init(self, world: &mut World) {
        world.insert_resource(BrickGrid::new(32));
        world.init_resource::<MaterialManager>();

        world.add_systems(Startup, (create_materials, generate).chain());
    }
}

#[derive(Debug, Clone, Resource)]
pub struct DefaultMaterials {
    pub dirt: MaterialId,
    pub grass: MaterialId,
    pub stone: MaterialId,
    pub sand: MaterialId,
    pub snow: MaterialId,
    pub water: MaterialId,
    pub gravel: MaterialId,
}

fn create_materials(mut manager: ResMut<MaterialManager>, mut commands: Commands) {
    commands.insert_resource(DefaultMaterials {
        dirt: manager.register(Material::new(Vec3::new(0.55, 0.36, 0.17), 0.0)),
        grass: manager.register(Material::new(Vec3::new(0.30, 0.65, 0.15), 0.0)),
        stone: manager.register(Material::new(Vec3::new(0.50, 0.50, 0.52), 0.0)),
        sand: manager.register(Material::new(Vec3::new(0.90, 0.85, 0.60), 0.0)),
        snow: manager.register(Material::new(Vec3::new(0.95, 0.96, 0.98), 0.0)),
        water: manager.register(Material::new(Vec3::new(0.6, 0.8, 1.0), 0.25)),
        gravel: manager.register(Material::new(Vec3::new(0.60, 0.58, 0.55), 0.0)),
    });
}

fn generate(
    materials: Res<DefaultMaterials>,
    mut grid: ResMut<BrickGrid>,
    mut world: ResMut<GpuWorld>,
) {
    let fbm: Fbm<Perlin> = Fbm::new(128)
        .set_frequency(0.005)
        .set_octaves(4)
        .set_lacunarity(2.0)
        .set_persistence(0.5);

    let brick_size = BrickMap::SIZE as i32;
    let total_size = grid.size as i32 * brick_size;
    let half = total_size / 2;
    let amplitude = total_size as f64 * 0.25;

    let sea_level = 0_i32;
    let snow_line = (amplitude * 0.7) as i32;

    for x in -half..half {
        for z in -half..half {
            let height = fbm.get([x as f64, z as f64]) * amplitude;
            let max_y = height.round() as i32;

            for y in -half..max_y.min(half) {
                let depth_from_surface = max_y - y;

                let mat = if max_y >= snow_line {
                    // Snow-capped peaks
                    if depth_from_surface == 1 {
                        materials.snow
                    } else if depth_from_surface < 5 {
                        materials.stone
                    } else {
                        materials.stone
                    }
                } else if max_y <= sea_level + 2 && max_y >= sea_level - 3 {
                    // Beach / shoreline
                    if depth_from_surface < 4 {
                        materials.sand
                    } else {
                        materials.stone
                    }
                } else if max_y < sea_level - 3 {
                    // Underwater floor
                    if depth_from_surface < 3 {
                        materials.gravel
                    } else {
                        materials.stone
                    }
                } else {
                    // Normal terrain
                    if depth_from_surface == 1 {
                        materials.grass
                    } else if depth_from_surface < 5 {
                        materials.dirt
                    } else {
                        materials.stone
                    }
                };

                let edit = grid.set_voxel(ivec3(x, y, z), mat);
                world.apply(edit);
            }

            // Fill water up to sea level
            if max_y < sea_level {
                for y in max_y..sea_level {
                    let edit = grid.set_voxel(ivec3(x, y, z), materials.water);
                    world.apply(edit);
                }
            }
        }
    }

    let mut total = 0_usize;
    for brick in &grid.grid {
        total += brick.count() as usize;
    }

    info!("total {total}");
}
