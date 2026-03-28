use std::collections::HashMap;

use bevy_ecs::resource::Resource;
use encase::ShaderType;

use crate::{Edit, SlotAllocator};

pub const UNPACK_OP_CLEAR: u32 = 0;
pub const UNPACK_OP_UNLOAD: u32 = 1;
pub const UNPACK_OP_SET: u32 = 2;

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct UnpackCommand {
    pub grid_idx: u32,
    pub map_idx: u32,
    pub op: u32,
    pub data: [u32; 16],
}

impl UnpackCommand {
    fn set(grid_idx: u32, map_idx: u32, data: [u32; 16]) -> Self {
        UnpackCommand {
            grid_idx,
            map_idx,
            op: UNPACK_OP_SET,
            data,
        }
    }

    fn clear(grid_idx: u32, map_idx: u32) -> Self {
        UnpackCommand {
            grid_idx,
            map_idx,
            op: UNPACK_OP_CLEAR,
            data: [0; 16],
        }
    }
}

#[derive(Debug, Default, Resource)]
pub struct GpuWorld {
    allocator: SlotAllocator,

    pending: HashMap<u32, UnpackCommand>,
    grid_map: HashMap<u32, u32>,
}

impl GpuWorld {
    pub fn new(size: usize) -> Self {
        Self {
            allocator: SlotAllocator::new(size),
            pending: HashMap::new(),
            grid_map: HashMap::new(),
        }
    }

    pub fn apply(&mut self, edit: Edit) {
        match edit {
            Edit::Set { grid_idx, brick } => {
                let grid_idx = grid_idx as u32;

                let map_idx = *self
                    .grid_map
                    .entry(grid_idx)
                    .or_insert_with(|| self.allocator.alloc().expect("no free map slots") as u32);

                self.pending
                    .insert(grid_idx, UnpackCommand::set(grid_idx, map_idx, brick.mask));
            }
            Edit::Clear { grid_idx } => {
                let grid_idx = grid_idx as u32;

                if let Some(map_idx) = self.grid_map.remove(&grid_idx) {
                    self.allocator.free(map_idx as usize);
                    self.pending
                        .insert(grid_idx, UnpackCommand::clear(grid_idx, map_idx));
                }
            }
        }
    }

    pub fn drain_commands(&mut self) -> (usize, impl Iterator<Item = UnpackCommand>) {
        let len = self.pending.len();
        (len, self.pending.drain().map(|(_, cmd)| cmd))
    }

    pub fn map_capacity(&self) -> u64 {
        self.allocator.capacity as u64
    }
}
