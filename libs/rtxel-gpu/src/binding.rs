use std::{marker::PhantomData, num::NonZero};

use wgpu::{BindingResource, BindingType, Buffer, BufferBindingType, BufferSize};

use crate::{Bind, ImplicitBindGroupLayoutEntry};

fn min_binding_size<T>() -> BufferSize {
    NonZero::new(size_of::<T>() as u64).expect("buffer can't store ZST")
}

pub struct StorageBuffer<const READ_ONLY: bool, T>(PhantomData<T>);

pub type ReadStorageBufer<T> = StorageBuffer<true, T>;
pub type WriteStorageBuffer<T> = StorageBuffer<false, T>;

impl<const READ_ONLY: bool, T> Bind for StorageBuffer<READ_ONLY, T> {
    type Resource = Buffer;

    fn layout() -> ImplicitBindGroupLayoutEntry {
        ImplicitBindGroupLayoutEntry {
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage {
                    read_only: READ_ONLY,
                },
                has_dynamic_offset: false,
                min_binding_size: Some(min_binding_size::<T>()),
            },
            count: None,
        }
    }

    fn resource<'res>(resource: &'res Self::Resource) -> BindingResource<'res> {
        resource.as_entire_binding()
    }
}

pub struct UniformBuffer<T>(PhantomData<T>);

impl<T> Bind for UniformBuffer<T> {
    type Resource = Buffer;

    fn layout() -> ImplicitBindGroupLayoutEntry {
        ImplicitBindGroupLayoutEntry {
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(min_binding_size::<T>()),
            },
            count: None,
        }
    }

    fn resource<'res>(resource: &'res Self::Resource) -> BindingResource<'res> {
        resource.as_entire_binding()
    }
}
