/// Simple slot allocator used for managing GPU allocations
#[derive(Debug, Default, Clone)]
pub struct SlotAllocator {
    capacity: usize,
    size: usize,
    free: Vec<usize>,
}

impl SlotAllocator {
    /// Creates a empty slot allocator
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a allocator with given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            size: 0,
            free: (0..capacity).rev().collect(),
        }
    }

    /// Allocate a fresh slot
    pub fn alloc(&mut self) -> usize {
        self.size += 1;
        if self.size > self.capacity {
            todo!("proper handling of this")
        }

        self.free.pop().unwrap()
    }

    /// Free a slot
    pub fn free(&mut self, slot: usize) {
        assert!(
            slot < self.capacity,
            "slot {slot} is out of allocator range"
        );
        self.free.push(slot);
    }
}
