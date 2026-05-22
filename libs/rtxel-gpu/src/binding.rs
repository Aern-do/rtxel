use std::{marker::PhantomData, num::NonZero};

use wgpu::{
    BindingResource, BindingType, Buffer, BufferBindingType, BufferSize, Sampler,
    SamplerBindingType, StorageTextureAccess, TextureFormat, TextureSampleType, TextureView,
};

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

pub trait AsTextureSampleType {
    fn texture_sample_type() -> TextureSampleType;
}

pub struct Float<const FILTERABLE: bool>;

pub type FloatFilterable = Float<true>;
pub type FloatNonFilterable = Float<false>;

impl<const FILTERABLE: bool> AsTextureSampleType for Float<FILTERABLE> {
    fn texture_sample_type() -> TextureSampleType {
        TextureSampleType::Float {
            filterable: FILTERABLE,
        }
    }
}

pub struct Depth;

impl AsTextureSampleType for Depth {
    fn texture_sample_type() -> TextureSampleType {
        TextureSampleType::Depth
    }
}

pub struct SInt;

impl AsTextureSampleType for SInt {
    fn texture_sample_type() -> TextureSampleType {
        TextureSampleType::Sint
    }
}

pub struct UInt;

impl AsTextureSampleType for UInt {
    fn texture_sample_type() -> TextureSampleType {
        TextureSampleType::Uint
    }
}

pub struct Texture2D<S>(PhantomData<S>);

impl<S: AsTextureSampleType> Bind for Texture2D<S> {
    type Resource = TextureView;

    fn layout() -> ImplicitBindGroupLayoutEntry {
        ImplicitBindGroupLayoutEntry {
            ty: BindingType::Texture {
                sample_type: S::texture_sample_type(),
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }
    }

    fn resource<'res>(resource: &'res Self::Resource) -> BindingResource<'res> {
        BindingResource::TextureView(resource)
    }
}

pub trait AsSamplerBindingType {
    fn sampler_binding_type() -> SamplerBindingType;
}

pub struct Filtering;

impl AsSamplerBindingType for Filtering {
    fn sampler_binding_type() -> SamplerBindingType {
        SamplerBindingType::Filtering
    }
}

pub struct NonFiltering;

impl AsSamplerBindingType for NonFiltering {
    fn sampler_binding_type() -> SamplerBindingType {
        SamplerBindingType::NonFiltering
    }
}

pub struct Comparison;

impl AsSamplerBindingType for Comparison {
    fn sampler_binding_type() -> SamplerBindingType {
        SamplerBindingType::Comparison
    }
}

pub struct TextureSampler<S>(PhantomData<S>);

impl<S: AsSamplerBindingType> Bind for TextureSampler<S> {
    type Resource = Sampler;

    fn layout() -> ImplicitBindGroupLayoutEntry {
        ImplicitBindGroupLayoutEntry {
            ty: BindingType::Sampler(S::sampler_binding_type()),
            count: None,
        }
    }

    fn resource<'res>(sampler: &'res Self::Resource) -> BindingResource<'res> {
        BindingResource::Sampler(sampler)
    }
}

pub trait AsTextureFormat {
    fn texture_format() -> TextureFormat;
}

pub struct Rgba8Unorm;

impl AsTextureFormat for Rgba8Unorm {
    fn texture_format() -> TextureFormat {
        TextureFormat::Rgba8Unorm
    }
}

pub struct Rgba32Float;

impl AsTextureFormat for Rgba32Float {
    fn texture_format() -> TextureFormat {
        TextureFormat::Rgba32Float
    }
}

pub trait AsStorageTextureAccess {
    fn as_storage_texture_access() -> StorageTextureAccess;
}

pub struct Read;

impl AsStorageTextureAccess for Read {
    fn as_storage_texture_access() -> StorageTextureAccess {
        StorageTextureAccess::ReadOnly
    }
}

pub struct Write;

impl AsStorageTextureAccess for Write {
    fn as_storage_texture_access() -> StorageTextureAccess {
        StorageTextureAccess::WriteOnly
    }
}

pub struct ReadWrite;

impl AsStorageTextureAccess for ReadWrite {
    fn as_storage_texture_access() -> StorageTextureAccess {
        StorageTextureAccess::ReadWrite
    }
}

pub struct StorageTexture<S, F>(PhantomData<(S, F)>);
pub type RStorageTexture<F> = StorageTexture<Read, F>;
pub type WStorageTexture<F> = StorageTexture<Write, F>;
pub type RWStorageTexture<F> = StorageTexture<ReadWrite, F>;

impl<S: AsStorageTextureAccess, F: AsTextureFormat> Bind for StorageTexture<S, F> {
    type Resource = TextureView;

    fn layout() -> ImplicitBindGroupLayoutEntry {
        ImplicitBindGroupLayoutEntry {
            ty: BindingType::StorageTexture {
                access: S::as_storage_texture_access(),
                format: F::texture_format(),
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        }
    }

    fn resource<'res>(resource: &'res Self::Resource) -> BindingResource<'res> {
        BindingResource::TextureView(resource)
    }
}
