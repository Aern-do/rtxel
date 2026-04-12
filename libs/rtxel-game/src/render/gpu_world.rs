use std::collections::HashMap;

use bevy_ecs::resource::Resource;
use encase::ShaderType;

use crate::{Edit, SlotAllocator, UnpackCommand};

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct GpuBrickMap {
    pub mask: [u32; 16],
    pub material_idx: u32,
    pub is_requested: u32,
}

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct GpuPallete {
    pub pallete: [u32; 512],
}

#[derive(Debug, Default, Resource)]
pub struct GpuWorld {
    allocator: SlotAllocator,
    pallete_allocator: SlotAllocator,

    pending: HashMap<u32, UnpackCommand>,
    grid_map: HashMap<u32, u32>,
    map_pallete: HashMap<u32, u32>,
}

impl GpuWorld {
    pub fn new(size: usize) -> Self {
        Self {
            allocator: SlotAllocator::new(size),
            pallete_allocator: SlotAllocator::new(4),
            pending: HashMap::new(),
            grid_map: HashMap::new(),
            map_pallete: HashMap::new(),
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

                let command = match brick.uniform_material() {
                    Some(material) => UnpackCommand::SetUniform {
                        grid_idx,
                        map_idx,
                        material_idx: material.id() as u32,
                        mask: brick.mask,
                    },
                    None => {
                        let pallete_idx = *self.map_pallete.entry(map_idx).or_insert_with(|| {
                            self.pallete_allocator.alloc().expect("no pallete slots") as u32
                        });

                        self.map_pallete.insert(map_idx, pallete_idx as u32);

                        UnpackCommand::SetPallete {
                            grid_idx,
                            map_idx,
                            pallete_idx: pallete_idx as u32,
                            mask: brick.mask,
                            pallete: brick.materials.map(|material| material.id() as u32),
                        }
                    }
                };

                self.pending.insert(grid_idx, command);
            }
            Edit::Clear { grid_idx } => {
                let grid_idx = grid_idx as u32;

                if let Some(map_idx) = self.grid_map.remove(&grid_idx) {
                    self.allocator.free(map_idx as usize);
                    if let Some(pallete_idx) = self.map_pallete.get(&map_idx) {
                        self.pallete_allocator.free(*pallete_idx as usize)
                    }
                    self.pending
                        .insert(grid_idx, UnpackCommand::Clear { grid_idx });
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

    pub fn pallete_capacity(&self) -> u64 {
        self.pallete_allocator.capacity as u64
    }
}
