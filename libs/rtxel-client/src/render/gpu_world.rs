use std::collections::HashMap;

use bytemuck::NoUninit;
use rtxel_gpu::Ctx;
use wgpu::{Buffer, BufferUsages};

use crate::{
    render::{Command, SlotAllocator},
    world::{Edit, Map, Map2, Map4, Map8, World},
};

pub type GPUMap8 = [u32; Map8::WORDS];
pub type GPUMap4 = [u32; Map4::WORDS];
pub type GPUMap2 = [u32; Map2::WORDS];

/// Manages grid and map buffers
#[derive(Debug, Clone)]
pub struct GPUWorld {
    pub grid_buffer: Buffer,
    pub map8_buffer: Buffer,
    pub map4_buffer: Buffer,
    pub map2_buffer: Buffer,

    pub allocator: SlotAllocator,
    pub pending: HashMap<u32, Command>,
    pub grid_to_map: HashMap<u32, u32>,
}

impl GPUWorld {
    pub fn new(ctx: &Ctx, world: &World) -> Self {
        let grid_buffer = ctx.create_buffer::<u32>(
            world.grid_volume(),
            Some("Grid Buffer"),
            BufferUsages::STORAGE,
        );

        let map8_buffer = Self::create_map_buffer::<GPUMap8>(ctx, &world);
        let map4_buffer = Self::create_map_buffer::<GPUMap4>(ctx, &world);
        let map2_buffer = Self::create_map_buffer::<GPUMap2>(ctx, &world);

        Self {
            grid_buffer,
            map8_buffer,
            map4_buffer,
            map2_buffer,
            allocator: SlotAllocator::with_capacity(world.grid_volume()),

            pending: HashMap::default(),
            grid_to_map: HashMap::default(),
        }
    }

    pub fn apply(&mut self, edit: Edit) {
        match edit {
            Edit::Set {
                grid,
                map8,
                map4,
                map2,
            } => {
                let grid = grid as u32;
                let map = self.acquire_map(grid);

                self.pending.insert(
                    grid,
                    Command::set(
                        grid,
                        map,
                        map8.max_height_y(),
                        map8.mask,
                        map4.mask,
                        map2.mask,
                    ),
                );
            }
            Edit::Clear { grid } => {
                let grid = grid as u32;

                self.free_map(grid);
                self.pending.insert(grid, Command::clear(grid));
            }
        };
    }

    pub fn drain_commands(&mut self) -> impl Iterator<Item = Command> {
        self.pending.drain().map(|(_, cmd)| cmd)
    }

    fn acquire_map(&mut self, grid: u32) -> u32 {
        *self
            .grid_to_map
            .entry(grid)
            .or_insert_with(|| self.allocator.alloc() as u32)
    }

    fn free_map(&mut self, grid: u32) {
        if let Some(map) = self.grid_to_map.remove(&grid) {
            self.allocator.free(map as usize);
        };
    }

    fn create_map_buffer<T: NoUninit>(ctx: &Ctx, world: &World) -> Buffer {
        ctx.create_buffer::<T>(
            world.grid_volume(),
            Some("Map Buffer"),
            BufferUsages::STORAGE,
        )
    }
}
