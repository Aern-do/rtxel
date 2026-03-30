use bevy_ecs::resource::Resource;
use glam::{IVec3, USizeVec3, UVec3};

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

    pub fn set_voxel(&mut self, pos: IVec3) -> Edit {
        let brick_size = BrickMap::SIZE as i32;

        let brick_pos = pos.div_euclid(IVec3::splat(brick_size));
        let local_pos = pos.rem_euclid(IVec3::splat(brick_size)).as_uvec3();

        let grid_idx = self.brick_idx(brick_pos);
        let brick = &mut self.grid[grid_idx];
        brick.set(local_pos);

        Edit::Set {
            grid_idx,
            brick: *brick,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BrickMap {
    pub mask: [u32; Self::WORDS],
    pub is_dirty: bool,
}

impl BrickMap {
    pub const SIZE: usize = 8;
    const WORDS: usize = Self::SIZE.pow(3) / 32;

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

    pub fn set(&mut self, pos: UVec3) {
        self.is_dirty = true;

        let (word, bit) = Self::index(pos);
        self.mask[word] |= bit;
    }

    pub fn clear(&mut self, pos: UVec3) {
        self.is_dirty = true;

        let (word, bit) = Self::index(pos);
        self.mask[word] &= !bit;
    }

    pub fn is_empty(&self) -> bool {
        self.mask.iter().all(|&w| w == 0)
    }
}
