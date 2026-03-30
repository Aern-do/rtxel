use std::{borrow::Cow, marker::PhantomData};

use encase::{ShaderType, StorageBuffer, internal::WriteInto};
use log::info;
use wgpu::{Buffer, BufferUsages, CommandEncoder};

use crate::Ctx;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DynamicBufferKind {
    Storage,
    Uniform,
}

#[derive(Debug, Clone)]
pub struct DynamicBufferDescriptor<'lbl> {
    pub label: Option<Cow<'lbl, str>>,
    pub usage: BufferUsages,
    pub kind: DynamicBufferKind,
}

#[derive(Debug)]
pub struct DynamicBuffer<T> {
    descriptor: DynamicBufferDescriptor<'static>,
    buffer: Buffer,
    capacity: u64,
    size: u64,
    _marker: PhantomData<T>,
}

impl<T: ShaderType + WriteInto> DynamicBuffer<T> {
    const INITIAL_CAPCITY: u64 = 4;
    const GROWTH_FACTOR: f32 = 1.5;

    fn create_buffer(descriptor: &DynamicBufferDescriptor, capacity: u64, ctx: &Ctx) -> Buffer {
        ctx.buffer(
            descriptor.label.as_deref(),
            T::min_size().get() * capacity,
            descriptor.usage | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        )
    }

    // maybe using fucking encase wasnt a best idea
    // but its too late now
    fn serialize_elements(&self, elements: &[T]) -> Vec<u8> {
        let stride = T::min_size().get() as usize;
        let mut bytes = Vec::with_capacity(stride * elements.len());

        for element in elements {
            let mut tmp = StorageBuffer::new(Vec::with_capacity(stride));
            tmp.write(element).expect("failed to serialize element");

            bytes.extend_from_slice(&tmp.into_inner());
        }

        bytes
    }

    pub fn new(descriptor: DynamicBufferDescriptor, ctx: &Ctx) -> Self {
        let buffer = Self::create_buffer(&descriptor, Self::INITIAL_CAPCITY, ctx);

        Self {
            descriptor: DynamicBufferDescriptor {
                label: descriptor.label.map(|label| label.into_owned().into()),
                ..descriptor
            },
            size: 0,
            capacity: Self::INITIAL_CAPCITY,
            buffer,
            _marker: PhantomData,
        }
    }

    pub fn ensure_capacity(
        &mut self,
        required: u64,
        encoder: &mut CommandEncoder,
        ctx: &Ctx,
    ) -> bool {
        if required <= self.capacity {
            return false;
        }

        let old_capacity = self.capacity;

        let growth = (Self::GROWTH_FACTOR as f64).ln();
        let steps = ((required as f64 / self.capacity as f64).ln() / growth).ceil() as u32;
        let new_capacity =
            (self.capacity as f64 * (Self::GROWTH_FACTOR as f64).powi(steps as i32)) as u64;

        self.capacity = new_capacity;

        let new_buffer = Self::create_buffer(&self.descriptor, self.capacity, ctx);
        encoder.copy_buffer_to_buffer(
            &self.buffer,
            0,
            &new_buffer,
            0,
            old_capacity * T::min_size().get(),
        );
        self.buffer = new_buffer;

        if let Some(label) = &self.descriptor.label {
            info!("buffer \"{label}\" resized from {old_capacity} to {new_capacity}",)
        } else {
            info!("buffer resized from {old_capacity} to {new_capacity}",)
        };

        true
    }

    pub fn write_iter(
        &mut self,
        iter: impl Iterator<Item = T>,
        encoder: &mut CommandEncoder,
        ctx: &Ctx,
    ) {
        // 1 allocations is much better than doing write_buffer multiple times
        let elements = iter.collect::<Vec<_>>();
        self.write(&elements, encoder, ctx);
    }

    pub fn write(&mut self, elements: &[T], encoder: &mut CommandEncoder, ctx: &Ctx) {
        let count = elements.len() as u64;
        self.ensure_capacity(count, encoder, ctx);

        let bytes = self.serialize_elements(elements);
        ctx.queue.write_buffer(&self.buffer, 0, &bytes);
        self.size = count;
    }

    pub fn push_iter(
        &mut self,
        iter: impl Iterator<Item = T>,
        encoder: &mut CommandEncoder,
        ctx: &Ctx,
    ) {
        let elements = iter.collect::<Vec<_>>();
        self.push(&elements, encoder, ctx);
    }

    pub fn push(&mut self, elements: &[T], encoder: &mut CommandEncoder, ctx: &Ctx) {
        let count = elements.len() as u64;
        self.ensure_capacity(self.size + count, encoder, ctx);

        let bytes = self.serialize_elements(elements);
        let offset = self.size * T::min_size().get();
        ctx.queue.write_buffer(&self.buffer, offset, &bytes);
        self.size += count;
    }

    pub fn clear(&mut self) {
        self.size = 0;
    }

    pub fn mark_fully_used(&mut self) {
        self.size = self.capacity;
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}
