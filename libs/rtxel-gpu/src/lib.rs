use std::{borrow::Cow, sync::Mutex};

use bevy_ecs::resource::Resource;
use wgpu::{
    Backends, BindGroupLayout, Buffer, BufferUsages, Device, Extent3d, Instance,
    InstanceDescriptor, PipelineLayout, PipelineLayoutDescriptor, PowerPreference, PresentMode,
    Queue, RequestAdapterOptions, ShaderModule, ShaderModuleDescriptor, ShaderSource, Surface,
    SurfaceConfiguration, SurfaceTarget, Texture, TextureDimension, TextureFormat, TextureUsages,
    wgt::{BufferDescriptor, DeviceDescriptor, TextureDescriptor},
};

pub mod bind_group;
pub mod binding;
pub mod compute_pipeline;
pub mod dynamic_buffer;
pub mod render_pipeline;

pub use bind_group::*;
pub use binding::*;
pub use compute_pipeline::*;
pub use dynamic_buffer::*;
pub use render_pipeline::*;

#[derive(Debug, Resource)]
pub struct Ctx {
    pub device: Device,
    pub queue: Queue,
    pub config: Mutex<SurfaceConfiguration>,
    pub surface: Surface<'static>,
}

impl Ctx {
    pub async fn new(target: impl Into<SurfaceTarget<'static>>, width: u32, height: u32) -> Self {
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::from_env().unwrap_or(Backends::PRIMARY),
            ..Default::default()
        });

        let surface = instance
            .create_surface(target)
            .expect("failed to create surface");

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("failed to get adapter");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .await
            .expect("failed to get device");

        let mut config = surface
            .get_default_config(&adapter, width, height)
            .expect("failed to get default surface config");

        config.present_mode = PresentMode::AutoNoVsync;
        surface.configure(&device, &config);

        Self {
            device,
            queue,
            config: Mutex::new(config),
            surface,
        }
    }

    pub fn size(&self) -> (u32, u32) {
        let config = self.config.lock().expect("failed to lock config");
        (config.width, config.height)
    }

    pub fn wgsl_shader(&self, label: Option<&str>, source: &str) -> ShaderModule {
        self.device.create_shader_module(ShaderModuleDescriptor {
            label,
            source: ShaderSource::Wgsl(Cow::Borrowed(source)),
        })
    }

    pub fn pipeline_layout(
        &self,
        label: Option<&str>,
        bind_group_layouts: &[&BindGroupLayout],
    ) -> PipelineLayout {
        self.device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label,
                bind_group_layouts,
                immediate_size: 0,
            })
    }

    pub fn render_pipeline<'ctx, 'pl>(
        &'ctx self,
        base: BasePipeline<'pl>,
    ) -> RenderPipelineBuilder<'ctx, 'pl> {
        RenderPipelineBuilder::new(base, self)
    }

    pub fn compute_pipeline<'ctx, 'pl>(
        &'ctx self,
        base: BaseComputePipeline<'pl>,
    ) -> ComputePipelineBuilder<'ctx, 'pl> {
        ComputePipelineBuilder::new(base, self)
    }

    pub fn buffer(&self, label: Option<&str>, size: u64, usage: BufferUsages) -> Buffer {
        self.device.create_buffer(&BufferDescriptor {
            label,
            size,
            usage,
            mapped_at_creation: false,
        })
    }

    pub fn texture(
        &self,
        label: Option<&str>,
        width: u32,
        height: u32,
        format: TextureFormat,
        usage: TextureUsages,
    ) -> Texture {
        self.device.create_texture(&TextureDescriptor {
            label,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    }
}
