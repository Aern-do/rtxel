use glam::{IVec3, USizeVec3};

use crate::world::{Map8, flatten};

/// Dynamically sized volume of [maps](Map)
#[derive(Debug, Clone)]
pub struct Grid {
    pub size: USizeVec3,
    pub grid: Box<[Map8]>,
}

impl Grid {
    /// Create a grid with given size
    pub fn new(size: USizeVec3) -> Self {
        Self {
            size,
            grid: vec![Map8::default(); size.element_product()].into_boxed_slice(),
        }
    }

    /// Get a map index given global brick position
    pub fn idx(&self, pos: IVec3) -> usize {
        let half_size = self.size.as_ivec3() / 2;
        let pos = (pos + half_size).as_usizevec3();

        flatten(pos, self.size)
    }

    /// Get a map reference given global brick position
    pub fn map(&self, pos: IVec3) -> &Map8 {
        let idx = self.idx(pos);
        &self.grid[idx]
    }

    /// Get a mutable map reference given global voxel position
    pub fn map_mut(&mut self, pos: IVec3) -> &mut Map8 {
        let idx = self.idx(pos);
        &mut self.grid[idx]
    }
}
