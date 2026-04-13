use bevy_ecs::resource::Resource;
use glam::{IVec3, USizeVec3, UVec3};

use crate::MaterialId;

fn flatten(pos: USizeVec3, size: usize) -> usize {
    pos.x + pos.y * size + pos.z * size * size
}

#[derive(Debug, Clone)]
pub enum Edit {
    Set { grid_idx: usize, brick: BrickMap },
    Clear { grid_idx: usize },
}

#[derive(Debug, Clone, Resource)]
pub struct BrickGrid {
    pub size: usize,
    pub grid: Vec<BrickMap>,
}

impl BrickGrid {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            grid: vec![BrickMap::new(); size.pow(3)],
        }
    }

    pub fn brick_idx(&self, pos: IVec3) -> usize {
        let half_size = self.size as i32 / 2;
        let pos = (pos + half_size).as_usizevec3();

        flatten(pos, self.size)
    }

    pub fn brick(&self, pos: IVec3) -> &BrickMap {
        let idx = self.brick_idx(pos);
        &self.grid[idx]
    }

    pub fn brick_mut(&mut self, pos: IVec3) -> &mut BrickMap {
        let idx = self.brick_idx(pos);
        &mut self.grid[idx]
    }

    pub fn set_voxel(&mut self, pos: IVec3, material: MaterialId) -> Edit {
        let brick_size = BrickMap::SIZE as i32;

        let brick_pos = pos.div_euclid(IVec3::splat(brick_size));
        let local_pos = pos.rem_euclid(IVec3::splat(brick_size)).as_uvec3();

        let grid_idx = self.brick_idx(brick_pos);
        let brick = &mut self.grid[grid_idx];
        if material == MaterialId::AIR {
            brick.clear(local_pos);
        } else {
            brick.set(local_pos, material);
        }

        Edit::Set {
            grid_idx,
            brick: *brick,
        }
    }

    fn brick_in_bounds(&self, brick_pos: IVec3) -> bool {
        let half = self.size as i32 / 2;
        brick_pos.cmpge(IVec3::splat(-half)).all() && brick_pos.cmplt(IVec3::splat(half)).all()
    }

    pub fn get_voxel(&self, pos: IVec3) -> Option<MaterialId> {
        let brick_size = BrickMap::SIZE as i32;
        let brick_pos = pos.div_euclid(IVec3::splat(brick_size));

        if !self.brick_in_bounds(brick_pos) {
            return None;
        }

        let local_pos = pos.rem_euclid(IVec3::splat(brick_size)).as_uvec3();
        let brick = self.brick(brick_pos);

        if brick.get(local_pos) {
            Some(brick.get_material(local_pos))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BrickMap {
    pub mask: [u32; Self::WORDS],
    pub materials: [MaterialId; Self::VOLUME],
}

impl Default for BrickMap {
    fn default() -> Self {
        Self {
            mask: [0; Self::WORDS],
            materials: [MaterialId::AIR; Self::VOLUME],
        }
    }
}

impl BrickMap {
    pub const SIZE: usize = 8;
    pub const VOLUME: usize = Self::SIZE.pow(3);

    const WORDS: usize = Self::VOLUME / 32;

    pub fn new() -> Self {
        Self::default()
    }

    fn index(pos: UVec3) -> (usize, u32) {
        let index = flatten(pos.as_usizevec3(), Self::SIZE);
        (index / 32, 1 << (index % 32))
    }

    pub fn count(&self) -> u32 {
        self.mask.iter().map(|w| w.count_ones()).sum()
    }

    pub fn get(&self, pos: UVec3) -> bool {
        let (word, bit) = Self::index(pos);
        self.mask[word] & bit != 0
    }

    pub fn get_material(&self, pos: UVec3) -> MaterialId {
        self.materials[flatten(pos.as_usizevec3(), Self::SIZE)]
    }

    pub fn set(&mut self, pos: UVec3, material: MaterialId) {
        let (word, bit) = Self::index(pos);
        self.mask[word] |= bit;
        self.materials[flatten(pos.as_usizevec3(), Self::SIZE)] = material;
    }

    pub fn clear(&mut self, pos: UVec3) {
        let (word, bit) = Self::index(pos);
        self.mask[word] &= !bit;
        self.materials[flatten(pos.as_usizevec3(), Self::SIZE)] = MaterialId::AIR;
    }

    pub fn is_empty(&self) -> bool {
        self.mask.iter().all(|&w| w == 0)
    }

    pub fn uniform_material(&self) -> Option<MaterialId> {
        let first = self.materials.iter().find(|&&m| m != MaterialId::AIR)?;
        self.materials
            .iter()
            .filter(|&&m| m != MaterialId::AIR)
            .all(|&m| m == *first)
            .then_some(*first)
    }
}
