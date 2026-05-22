pub mod bind_group;
pub mod binding;
pub mod compute_pipeline;
pub mod render_pipeline;
use std::{
    env,
    sync::{RwLock, RwLockReadGuard},
};

pub use bind_group::{AsBindGroup, Bind, ImplicitBindGroupLayoutEntry};
pub use binding::*;
use bytemuck::{NoUninit, checked::cast_slice};
pub use compute_pipeline::{BaseComputePipeline, ComputePipelineBuilder};
use log::info;
use wgpu::{
    BackendOptions, Backends, Buffer, BufferUsages, Device, Features, Instance, InstanceDescriptor,
    InstanceFlags, Limits, MemoryBudgetThresholds, PowerPreference, PresentMode, Queue,
    RequestAdapterOptionsBase, Surface, SurfaceConfiguration, SurfaceTarget, Trace,
    util::{BufferInitDescriptor, DeviceExt},
    wgt::{BufferDescriptor, DeviceDescriptor},
};

use crate::render_pipeline::{BasePipeline, RenderPipelineBuilder};

#[derive(Debug)]
pub struct Ctx {
    pub device: Device,
    pub queue: Queue,
    pub config: RwLock<SurfaceConfiguration>,
    pub surface: Surface<'static>,
}

impl Ctx {
    pub async fn new(target: impl Into<SurfaceTarget<'static>>, width: u32, height: u32) -> Self {
        let trace_path = env::var("TRACE_PATH").ok();

        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::from_env().unwrap_or(Backends::PRIMARY),
            flags: InstanceFlags::from_env_or_default(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
            backend_options: BackendOptions::from_env_or_default(),
            display: None,
        });

        let surface = instance
            .create_surface(target)
            .expect("failed to create surface");

        let adapter = instance
            .request_adapter(&RequestAdapterOptionsBase {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("failed to get adapter");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                trace: trace_path
                    .map(|path| Trace::Directory(path.into()))
                    .unwrap_or_default(),
                required_limits: Limits {
                    max_buffer_size: 4096 * 1024 * 1024,
                    max_storage_buffer_binding_size: 4294967292,
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
            .expect("failed to get device");

        let mut config = surface
            .get_default_config(&adapter, width, height)
            .expect("failed to get default surface config");

        config.present_mode = PresentMode::AutoNoVsync;
        surface.configure(&device, &config);

        let info = adapter.get_info();
        info!(
            "initialized context for following backend: {}",
            info.backend
        );

        Self {
            device,
            queue,
            config: RwLock::new(config),
            surface,
        }
    }

    /// Returns current [surface configuration](wgpu::SurfaceConfiguration)
    pub fn config(&self) -> RwLockReadGuard<'_, SurfaceConfiguration> {
        self.config.read().expect("failed to read config")
    }

    /// Reconfigures surface using current [surface configuration](wgpu::SurfaceConfiguration)
    pub fn reconfigure(&self) {
        self.surface.configure(&self.device, &self.config());
    }

    /// Creates a [Buffer] with data to initalize it
    pub fn create_buffer_init<T: NoUninit>(
        &self,
        contents: &[T],
        label: Option<&str>,
        usage: BufferUsages,
    ) -> Buffer {
        self.device.create_buffer_init(&BufferInitDescriptor {
            label,
            contents: cast_slice(contents),
            usage,
        })
    }

    /// Creates a [Buffer] with given size
    pub fn create_buffer<T>(
        &self,
        size: usize,
        label: Option<&str>,
        usage: BufferUsages,
    ) -> Buffer {
        self.device.create_buffer(&BufferDescriptor {
            label,
            size: (size * size_of::<T>()) as u64,
            usage,
            mapped_at_creation: false,
        })
    }

    /// Create a compute pipeline builder with given base pipeline
    pub fn compute_pipeline<'ctx, 'pl>(
        &'ctx self,
        base: BaseComputePipeline<'pl>,
    ) -> ComputePipelineBuilder<'ctx, 'pl> {
        ComputePipelineBuilder::new(base, self)
    }

    pub fn render_pipeline<'ctx, 'pl>(
        &'ctx self,
        base: BasePipeline<'pl>,
    ) -> RenderPipelineBuilder<'ctx, 'pl> {
        RenderPipelineBuilder::new(base, self)
    }
}
