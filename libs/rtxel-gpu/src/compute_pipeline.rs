use wgpu::{
    ComputePipeline, ComputePipelineDescriptor, PipelineCompilationOptions, PipelineLayout,
    ShaderModule,
};

use crate::Ctx;

#[derive(Debug, Clone)]
pub struct BaseComputePipeline<'pl> {
    pub layout: &'pl PipelineLayout,
    pub shader: &'pl ShaderModule,
    pub entry: &'static str,
}

#[derive(Debug, Clone)]
pub struct ComputePipelineBuilder<'ctx, 'pl> {
    pub ctx: &'ctx Ctx,
    pub label: Option<&'pl str>,
    pub base: BaseComputePipeline<'pl>,
}

impl<'ctx, 'pl> ComputePipelineBuilder<'ctx, 'pl> {
    pub fn new(base: BaseComputePipeline<'pl>, ctx: &'ctx Ctx) -> Self {
        Self {
            ctx,
            label: None,
            base,
        }
    }

    pub fn label(mut self, label: &'pl str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn build(self) -> ComputePipeline {
        self.ctx
            .device
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: self.label,
                layout: Some(self.base.layout),
                module: self.base.shader,
                entry_point: Some(self.base.entry),
                compilation_options: PipelineCompilationOptions::default(),
                cache: None,
            })
    }
}
