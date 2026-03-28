use bevy_ecs::resource::Resource;
use encase::ShaderType;
use rtxel_gpu::{Ctx, DynamicBuffer, DynamicBufferDescriptor, DynamicBufferKind};
use wgpu::{BufferUsages, Texture, TextureFormat, TextureUsages};

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct GpuBrickMap {
    pub mask: [u32; 16],
    pub is_requested: u32,
}

#[derive(Debug, Resource)]
pub struct Resources {
    pub out_texture: Texture,
    pub grid_buffer: DynamicBuffer<u32>,
    pub map_buffer: DynamicBuffer<GpuBrickMap>,
}

impl Resources {
    pub fn new(width: u32, height: u32, ctx: &Ctx) -> Self {
        Self {
            out_texture: Self::create_out_texture(width, height, ctx),
            grid_buffer: DynamicBuffer::new(
                DynamicBufferDescriptor {
                    label: Some("Grid Dynamic Buffer".into()),
                    usage: BufferUsages::STORAGE,
                    kind: DynamicBufferKind::Storage,
                },
                ctx,
            ),
            map_buffer: DynamicBuffer::new(
                DynamicBufferDescriptor {
                    label: Some("Map Dynamic Buffer".into()),
                    usage: BufferUsages::STORAGE,
                    kind: DynamicBufferKind::Storage,
                },
                ctx,
            ),
        }
    }

    pub fn create_out_texture(width: u32, height: u32, ctx: &Ctx) -> Texture {
        ctx.texture(
            Some("Output Texture"),
            width,
            height,
            TextureFormat::Rgba8Unorm,
            TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        )
    }
}
