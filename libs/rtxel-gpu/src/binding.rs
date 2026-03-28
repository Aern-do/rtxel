use std::marker::PhantomData;

use encase::ShaderType;
use wgpu::{BindingResource, BindingType, Buffer, BufferBindingType};

use crate::bind_group::Bindable;

pub struct StorageBuffer<T, const W: bool>(PhantomData<T>);
pub type RWStorageBuffer<T> = StorageBuffer<T, true>;
pub type RStorageBuffer<T> = StorageBuffer<T, false>;

impl<T: ShaderType, const W: bool> Bindable for StorageBuffer<T, W> {
    type Resource = Buffer;

    fn binding_type() -> BindingType {
        BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: !W },
            has_dynamic_offset: false,
            min_binding_size: Some(T::min_size()),
        }
    }

    fn binding_resource(resource: &Self::Resource) -> BindingResource<'_> {
        resource.as_entire_binding()
    }
}

pub struct UniformBuffer<T>(PhantomData<T>);

impl<T: ShaderType> Bindable for UniformBuffer<T> {
    type Resource = Buffer;

    fn binding_type() -> BindingType {
        BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(T::min_size()),
        }
    }

    fn binding_resource(resource: &Self::Resource) -> BindingResource<'_> {
        resource.as_entire_binding()
    }
}
