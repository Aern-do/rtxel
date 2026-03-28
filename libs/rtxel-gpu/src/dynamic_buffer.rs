use std::{borrow::Cow, marker::PhantomData};

use encase::{ShaderType, StorageBuffer, UniformBuffer, internal::WriteInto};
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

    pub fn upload_iter(
        &mut self,
        iter: impl Iterator<Item = T>,
        encoder: &mut CommandEncoder,
        ctx: &Ctx,
    ) {
        // 1 allocations is much better than doing write_buffer multiple times
        let elements = iter.collect::<Vec<_>>();
        self.upload(&elements, encoder, ctx);
    }

    pub fn upload(&mut self, elements: &[T], encoder: &mut CommandEncoder, ctx: &Ctx) {
        self.ensure_capacity(self.size + elements.len() as u64, encoder, ctx);

        let bytes = match self.descriptor.kind {
            DynamicBufferKind::Storage => {
                let mut buffer = StorageBuffer::new(Vec::new());
                for element in elements {
                    buffer.write(element).expect("failed to seralize element");
                }
                buffer.into_inner()
            }
            DynamicBufferKind::Uniform => {
                let mut buffer = UniformBuffer::new(Vec::new());
                for element in elements {
                    buffer.write(element).expect("failed to seralize element");
                }
                buffer.into_inner()
            }
        };

        ctx.queue.write_buffer(&self.buffer, 0, &bytes);
        self.size += elements.len() as u64;
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
