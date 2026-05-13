use std::{env, sync::Mutex};

use log::info;
use wgpu::{
    BackendOptions, Backends, Device, Instance, InstanceDescriptor, InstanceFlags, Limits,
    MemoryBudgetThresholds, PowerPreference, PresentMode, Queue, RequestAdapterOptionsBase,
    Surface, SurfaceConfiguration, SurfaceTarget, Trace, wgt::DeviceDescriptor,
};

#[derive(Debug)]
pub struct Ctx {
    pub device: Device,
    pub queue: Queue,
    pub config: Mutex<SurfaceConfiguration>,
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
                required_limits: Limits::defaults(),
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
            config: Mutex::new(config),
            surface,
        }
    }
}
