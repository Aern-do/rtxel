use std::marker::PhantomData;

use encase::ShaderType;
use wgpu::{
    BindingResource, BindingType, Buffer, BufferBindingType, Sampler, SamplerBindingType,
    StorageTextureAccess, TextureFormat, TextureSampleType, TextureView, TextureViewDimension,
};

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

impl<S: AsStorageTextureAccess, F: AsTextureFormat> Bindable
    for StorageTexture<S, F>
{
    type Resource = TextureView;

    fn binding_type() -> BindingType {
        BindingType::StorageTexture {
            access: S::as_storage_texture_access(),
            format: F::texture_format(),
            view_dimension: TextureViewDimension::D2,
        }
    }

    fn binding_resource(resource: &Self::Resource) -> BindingResource<'_> {
        BindingResource::TextureView(resource)
    }
}

pub trait AsTextureSampleType {
    fn texture_sample_type() -> TextureSampleType;
}

pub struct Float<const FILTERABLE: bool>;

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

impl<S: AsTextureSampleType> Bindable for Texture2D<S> {
    type Resource = TextureView;

    fn binding_type() -> BindingType {
        BindingType::Texture {
            sample_type: S::texture_sample_type(),
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        }
    }

    fn binding_resource(resource: &Self::Resource) -> BindingResource<'_> {
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

impl<S: AsSamplerBindingType> Bindable for TextureSampler<S> {
    type Resource = Sampler;

    fn binding_type() -> BindingType {
        BindingType::Sampler(S::sampler_binding_type())
    }

    fn binding_resource(resource: &Self::Resource) -> BindingResource<'_> {
        BindingResource::Sampler(resource)
    }
}
