use glam::{IVec3, USizeVec3};

pub mod generator;
pub mod grid;
pub mod map;
pub use grid::Grid;
pub use map::{Map, Map2, Map4, Map8};

pub fn flatten(pos: USizeVec3, size: USizeVec3) -> usize {
    pos.x + pos.y * size.x + pos.z * size.x * size.y
}

/// Pending world edit that is going to converted to unpack command later in frame
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edit {
    /// Set a maps to given grid index
    Set {
        grid: usize,
        map8: Map8,
        map4: Map4,
        map2: Map2,
    },
    /// Clear grid at given index
    Clear { grid: usize },
}

/// Main world representation, holds grid and pending edits to world
#[derive(Debug, Clone)]
pub struct World {
    grid: Grid,
    pending_edits: Vec<Edit>,
}

impl World {
    /// Create a world with given size
    pub fn new(size: USizeVec3) -> Self {
        Self {
            grid: Grid::new(size),
            pending_edits: Vec::default(),
        }
    }

    /// Set a voxel at given global voxel position
    pub fn set_voxel(&mut self, pos: IVec3, state: bool) {
        let size = IVec3::splat(Map8::SIZE as i32);

        let brick_pos = pos.div_euclid(size);
        let local_pos = pos.rem_euclid(size).as_uvec3();

        let grid = self.grid.idx(brick_pos);
        let map8 = self.grid.map_mut(brick_pos);

        if state {
            map8.set(local_pos);
        } else {
            map8.clear(local_pos);
        }

        // no idea why it can't infer it itself
        let map4 = map8.downsample::<Map4>();
        let map2 = map4.downsample();

        self.pending_edits.push(Edit::Set {
            grid,
            map8: *map8,
            map4,
            map2,
        });
    }

    /// Drain pending edits
    pub fn drain_edits(&mut self) -> impl Iterator<Item = Edit> {
        self.pending_edits.drain(..)
    }

    /// Returns total volume of maps in world
    pub fn grid_volume(&self) -> usize {
        self.grid.size.element_product()
    }

    pub fn grid_size(&self) -> USizeVec3 {
        self.grid.size
    }
}
