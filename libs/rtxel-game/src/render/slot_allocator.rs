#[derive(Debug, Default, Clone)]
pub struct SlotAllocator {
    pub capacity: usize,
    pub size: usize,
    pub free: Vec<usize>,
}

impl SlotAllocator {
    pub fn new(slots: usize) -> Self {
        Self {
            size: 0,
            capacity: slots,
            free: (0..slots).rev().collect(),
        }
    }

    pub fn alloc(&mut self) -> Option<usize> {
        self.size += 1;
        if self.size > self.capacity {
            self.resize(self.capacity * 2);
        }

        self.free.pop()
    }

    pub fn free(&mut self, slot: usize) {
        assert!(
            slot < self.capacity,
            "slot {slot} is out of allocator range"
        );
        self.free.push(slot);
    }

    pub fn resize(&mut self, new_capacity: usize) {
        assert!(new_capacity > self.capacity, "new capacity must be higher");

        self.free.extend((self.capacity..new_capacity).rev());
        self.capacity = new_capacity;
    }
}
