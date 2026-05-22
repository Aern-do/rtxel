use glam::{USizeVec3, UVec3, uvec3};

use crate::world::flatten;

pub trait Map {
    const SIZE: u32;
    const VOLUME: u32 = Self::SIZE.pow(3);
    const WORDS: usize = Self::VOLUME.div_ceil(32) as usize;

    fn mask(&self) -> &[u32];
    fn mask_mut(&mut self) -> &mut [u32];

    /// Returns a index of a word in a mask for given index
    fn word(idx: usize) -> usize {
        idx / 32
    }

    /// Return a bitmask for a given index in a word
    fn bit(idx: usize) -> u32 {
        1 << (idx % 32)
    }

    /// Get a voxel at given local map position
    fn get(&self, pos: UVec3) -> bool {
        let idx = flatten(pos.as_usizevec3(), USizeVec3::splat(Self::SIZE as usize));
        self.mask()[Self::word(idx)] & (Self::bit(idx)) != 0
    }

    /// Set a voxel at given local map position
    fn set(&mut self, pos: UVec3) {
        let idx = flatten(pos.as_usizevec3(), USizeVec3::splat(Self::SIZE as usize));
        self.mask_mut()[Self::word(idx)] |= Self::bit(idx);
    }

    /// Clear a voxel at given local map position
    fn clear(&mut self, pos: UVec3) {
        let idx = flatten(pos.as_usizevec3(), USizeVec3::splat(Self::SIZE as usize));
        self.mask_mut()[Self::word(idx)] &= !(Self::bit(idx));
    }

    /// Check if map is completely empty
    fn is_empty(&self) -> bool {
        self.mask().iter().all(|&w| w == 0)
    }

    fn downsample<M: Map + Default>(&self) -> M {
        assert_eq!(
            M::SIZE * 2,
            Self::SIZE,
            "target map must be 2 times smaller"
        );

        let mut out = M::default();

        for z in 0..M::SIZE {
            for y in 0..M::SIZE {
                for x in 0..M::SIZE {
                    let mut count = 0u32;
                    for dz in 0..2 {
                        for dy in 0..2 {
                            for dx in 0..2 {
                                let cp = uvec3(x * 2 + dx, y * 2 + dy, z * 2 + dz);
                                if self.get(cp) {
                                    count += 1;
                                }
                            }
                        }
                    }
                    if count >= 4 {
                        out.set(uvec3(x, y, z));
                    }
                }
            }
        }
        out
    }
}
/// 8x8x8 volume of voxels represented compactly using a bitmask
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Map8 {
    pub mask: [u32; Self::WORDS],
}

impl Map8 {
    /// Creates a empty map
    pub fn new() -> Self {
        Self::default()
    }
}

impl Map for Map8 {
    const SIZE: u32 = 8;

    fn mask(&self) -> &[u32] {
        &self.mask
    }
    fn mask_mut(&mut self) -> &mut [u32] {
        &mut self.mask
    }
}

/// 4x4x4 volume of voxels represented compactly using a bitmask
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Map4 {
    pub mask: [u32; Self::WORDS],
}

impl Map for Map4 {
    const SIZE: u32 = 4;

    fn mask(&self) -> &[u32] {
        &self.mask
    }
    fn mask_mut(&mut self) -> &mut [u32] {
        &mut self.mask
    }
}

/// 2x2x2 volume of voxels represented compactly using a bitmask
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Map2 {
    pub mask: [u32; Self::WORDS],
}

impl Map for Map2 {
    const SIZE: u32 = 2;

    fn mask(&self) -> &[u32] {
        &self.mask
    }
    fn mask_mut(&mut self) -> &mut [u32] {
        &mut self.mask
    }
}
