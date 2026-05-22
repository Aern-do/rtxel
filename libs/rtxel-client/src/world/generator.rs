use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

use glam::IVec3;
use log::info;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin, RidgedMulti};
use rayon::prelude::*;

use crate::world::{Map, Map8, World};

const SEED: u32 = 1488;

const HEIGHT_SCALE: f64 = 120.0;

const CONTINENT_FREQ: f64 = 0.004;
const CONTINENT_AMP: f64 = 1.0;
const CONTINENT_OCTAVES: usize = 4;

const MOUNTAIN_FREQ: f64 = 0.004;
const MOUNTAIN_AMP: f64 = 1.8;
const MOUNTAIN_OCTAVES: usize = 5;

const MOUNTAIN_MASK_FREQ: f64 = 0.015;
const MOUNTAIN_MASK_THRESHOLD: f64 = 0.05;
const MOUNTAIN_MASK_SHARPNESS: f64 = 1.0;

const HILL_FREQ: f64 = 0.001;
const HILL_AMP: f64 = 0.3;
const HILL_OCTAVES: usize = 4;

const ROUGHNESS_FREQ: f64 = 0.08;
const ROUGHNESS_AMP: f64 = 0.02;

const BASE_LEVEL: f64 = -0.2;

const CHUNK_COLUMNS: usize = 4096;

pub fn generate(world: &mut World) {
    let half_x = (world.grid_size().x as i32 * Map8::SIZE as i32) / 2;
    let half_z = (world.grid_size().z as i32 * Map8::SIZE as i32) / 2;
    let half_y = (world.grid_size().y as i32 * Map8::SIZE as i32) / 2;

    let sampler = TerrainSampler::new(SEED);
    let size_x = (half_x * 2) as usize;
    let size_z = (half_z * 2) as usize;
    let total_columns = size_x * size_z;

    info!("generating world!");

    let start = Instant::now();
    let processed = AtomicUsize::new(0);
    let last_report = std::sync::Mutex::new(Instant::now());

    let mut chunk_start = 0usize;
    while chunk_start < total_columns {
        let chunk_end = (chunk_start + CHUNK_COLUMNS).min(total_columns);

        let heights: Vec<i32> = (chunk_start..chunk_end)
            .into_par_iter()
            .map(|i| {
                let x = (i % size_x) as i32 - half_x;
                let z = (i / size_x) as i32 - half_z;
                sampler.height_at(x, z, half_y).clamp(-half_y, half_y - 1)
            })
            .collect();

        for (offset, &height) in heights.iter().enumerate() {
            let i = chunk_start + offset;
            let x = (i % size_x) as i32 - half_x;
            let z = (i / size_x) as i32 - half_z;
            for y in -half_y..height {
                world.set_voxel(IVec3::new(x, y, z), true);
            }
        }

        let done = processed.fetch_add(chunk_end - chunk_start, Ordering::Relaxed)
            + (chunk_end - chunk_start);

        let mut last = last_report.lock().unwrap();
        if last.elapsed().as_millis() > 100 || done == total_columns {
            let pct = done as f64 / total_columns as f64 * 100.0;
            let elapsed = start.elapsed().as_secs_f64();
            let eta = if done > 0 {
                elapsed * (total_columns - done) as f64 / done as f64
            } else {
                0.0
            };

            info!(
                "[{:>5.1}%] {}/{} columns  elapsed {:.1}s  eta {:.1}s",
                pct, done, total_columns, elapsed, eta
            );
            *last = Instant::now();
        }

        chunk_start = chunk_end;
    }
}

struct TerrainSampler {
    continent: Fbm<Perlin>,
    mountains: RidgedMulti<Perlin>,
    mountain_mask: Fbm<Perlin>,
    hills: Fbm<Perlin>,
    roughness: Perlin,
}

impl TerrainSampler {
    fn new(seed: u32) -> Self {
        Self {
            continent: Fbm::<Perlin>::new(seed)
                .set_octaves(CONTINENT_OCTAVES)
                .set_frequency(1.0),
            mountains: RidgedMulti::<Perlin>::new(seed.wrapping_add(1))
                .set_octaves(MOUNTAIN_OCTAVES)
                .set_frequency(1.0),
            mountain_mask: Fbm::<Perlin>::new(seed.wrapping_add(2))
                .set_octaves(3)
                .set_frequency(1.0),
            hills: Fbm::<Perlin>::new(seed.wrapping_add(3))
                .set_octaves(HILL_OCTAVES)
                .set_frequency(1.0),
            roughness: Perlin::new(seed.wrapping_add(4)),
        }
    }

    fn height_at(&self, x: i32, z: i32, half_y: i32) -> i32 {
        let fx = x as f64;
        let fz = z as f64;

        let continent = self
            .continent
            .get([fx * CONTINENT_FREQ, fz * CONTINENT_FREQ])
            * CONTINENT_AMP;

        let ridged = self.mountains.get([fx * MOUNTAIN_FREQ, fz * MOUNTAIN_FREQ]);
        let smooth = self
            .continent
            .get([fx * MOUNTAIN_FREQ * 1.3, fz * MOUNTAIN_FREQ * 1.3]);
        let raw_mountains = (ridged * 0.6 + smooth * 0.4) * MOUNTAIN_AMP;

        let mask_raw = self
            .mountain_mask
            .get([fx * MOUNTAIN_MASK_FREQ, fz * MOUNTAIN_MASK_FREQ]);
        let mask = ((mask_raw - MOUNTAIN_MASK_THRESHOLD).max(0.0))
            .powf(MOUNTAIN_MASK_SHARPNESS)
            .min(1.0);

        let hills = self.hills.get([fx * HILL_FREQ, fz * HILL_FREQ]) * HILL_AMP;
        let roughness = self
            .roughness
            .get([fx * ROUGHNESS_FREQ, fz * ROUGHNESS_FREQ])
            * ROUGHNESS_AMP;

        let normalized = continent + raw_mountains * mask + hills + roughness;
        let base = BASE_LEVEL * half_y as f64;
        (base + normalized * HEIGHT_SCALE) as i32
    }
}
