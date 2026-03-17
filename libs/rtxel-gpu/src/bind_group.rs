use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry,
};

use crate::Ctx;

pub mod layout {
    use std::num::NonZeroU64;

    use wgpu::{
        BindGroupLayoutEntry, BindingType, BufferBindingType, SamplerBindingType, ShaderStages,
        StorageTextureAccess, Texture, TextureSampleType, TextureViewDimension,
    };

    pub fn texture(sample_type: TextureSampleType) -> BindingType {
        BindingType::Texture {
            sample_type,
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        }
    }

    pub fn texture_float() -> BindingType {
        texture(TextureSampleType::Float { filterable: true })
    }

    pub fn sampler_filtering() -> BindingType {
        BindingType::Sampler(SamplerBindingType::Filtering)
    }

    pub fn uniform_buffer(min_binding_size: Option<NonZeroU64>) -> BindingType {
        BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size,
        }
    }

    pub fn storage_buffer(read_only: bool, min_binding_size: Option<NonZeroU64>) -> BindingType {
        BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size,
        }
    }

    pub fn storage_texture(texture: &Texture, access: StorageTextureAccess) -> BindingType {
        BindingType::StorageTexture {
            access,
            format: texture.format(),
            view_dimension: TextureViewDimension::D2,
        }
    }

    pub fn r_storage_texture(texture: &Texture) -> BindingType {
        storage_texture(texture, StorageTextureAccess::ReadOnly)
    }

    pub fn w_storage_texture(texture: &Texture) -> BindingType {
        storage_texture(texture, StorageTextureAccess::WriteOnly)
    }

    pub fn rw_storage_texture(texture: &Texture) -> BindingType {
        storage_texture(texture, StorageTextureAccess::ReadWrite)
    }

    pub fn entry(binding: u32, ty: BindingType, visibility: ShaderStages) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding,
            visibility,
            ty,
            count: None,
        }
    }

    pub fn compute(binding: u32, ty: BindingType) -> BindGroupLayoutEntry {
        entry(binding, ty, ShaderStages::COMPUTE)
    }

    pub fn vertex(binding: u32, ty: BindingType) -> BindGroupLayoutEntry {
        entry(binding, ty, ShaderStages::VERTEX)
    }

    pub fn fragment(binding: u32, ty: BindingType) -> BindGroupLayoutEntry {
        entry(binding, ty, ShaderStages::FRAGMENT)
    }

    pub fn vertex_fragment(binding: u32, ty: BindingType) -> BindGroupLayoutEntry {
        entry(binding, ty, ShaderStages::VERTEX_FRAGMENT)
    }
}

// kinda useless
// just to match format of layout
pub mod binding {
    use wgpu::{BindGroupEntry, BindingResource, Buffer, Sampler, TextureView};

    pub fn view(view: &TextureView) -> BindingResource<'_> {
        BindingResource::TextureView(view)
    }

    pub fn sampler(sampler: &Sampler) -> BindingResource<'_> {
        BindingResource::Sampler(sampler)
    }

    pub fn buffer(buffer: &Buffer) -> BindingResource<'_> {
        buffer.as_entire_binding()
    }

    pub fn entry(binding: u32, resource: BindingResource) -> BindGroupEntry {
        BindGroupEntry { binding, resource }
    }
}

pub fn create_bind_group_layout(
    ctx: &Ctx,
    label: Option<&str>,
    entries: &[BindGroupLayoutEntry],
) -> BindGroupLayout {
    ctx.device
        .create_bind_group_layout(&BindGroupLayoutDescriptor { label, entries })
}

pub fn create_bind_group(
    ctx: &Ctx,
    label: Option<&str>,
    layout: &BindGroupLayout,
    entries: &[BindGroupEntry],
) -> BindGroup {
    ctx.device.create_bind_group(&BindGroupDescriptor {
        label,
        layout,
        entries,
    })
}
