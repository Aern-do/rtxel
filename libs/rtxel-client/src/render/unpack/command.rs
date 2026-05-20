use bytemuck::NoUninit;

use crate::world::{Map, Map2, Map4, Map8};

/// Command representing action for unpack shader to perform
#[derive(Debug, Clone, Copy, PartialEq, Eq, NoUninit)]
#[repr(C)]
pub struct Command {
    action: u32,
    grid: u32,
    map: u32,
    mask8: [u32; Map8::WORDS],
    mask4: [u32; Map4::WORDS],
    mask2: [u32; Map2::WORDS],
}

impl Command {
    const ACTION_CLEAR: u32 = 0;
    const ACTION_SET: u32 = 1;

    /// Create a clear command. Clears grid at given index.
    pub fn clear(grid: u32) -> Self {
        Self {
            action: Self::ACTION_CLEAR,
            grid,
            map: 0,
            mask8: [0; Map8::WORDS],
            mask4: [0; Map4::WORDS],
            mask2: [0; Map2::WORDS],
        }
    }

    /// Create a set command. Assigns map to a grid with given mask
    pub fn set(
        grid: u32,
        map: u32,
        mask8: [u32; Map8::WORDS],
        mask4: [u32; Map4::WORDS],
        mask2: [u32; Map2::WORDS],
    ) -> Self {
        Self {
            action: Self::ACTION_SET,
            grid,
            map,
            mask8,
            mask4,
            mask2,
        }
    }
}
