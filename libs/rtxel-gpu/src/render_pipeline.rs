use wgpu::{
    Color, ColorTargetState, ColorWrites, FragmentState, MultisampleState,
    PipelineCompilationOptions, PipelineLayout, PrimitiveState, PrimitiveTopology, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, TextureFormat, VertexState,
};

use crate::Ctx;

#[derive(Debug, Clone)]
pub struct BasePipeline<'pl> {
    pub layout: &'pl PipelineLayout,
    pub shader: &'pl ShaderModule,

    pub vertex_entry: &'static str,
    pub fragment_entry: &'static str,
    pub fragment_format: TextureFormat,
}

#[derive(Debug, Clone)]
pub struct RenderPipelineBuilder<'ctx, 'pl> {
    pub ctx: &'ctx Ctx,
    pub label: Option<&'pl str>,
    pub base: BasePipeline<'pl>,

    pub topology: PrimitiveTopology,
}

impl<'ctx, 'pl> RenderPipelineBuilder<'ctx, 'pl> {
    pub fn new(base: BasePipeline<'pl>, ctx: &'ctx Ctx) -> Self {
        Self {
            ctx,
            label: None,
            base,
            topology: PrimitiveTopology::TriangleList,
        }
    }

    pub fn label(mut self, label: &'pl str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn topology(mut self, topolgy: PrimitiveTopology) -> Self {
        self.topology = topolgy;
        self
    }

    pub fn build(self) -> RenderPipeline {
        self.ctx.device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: self.label,
                layout: Some(self.base.layout),
                vertex: VertexState {
                    module: self.base.shader,
                    entry_point: Some(self.base.vertex_entry),
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                fragment: Some(FragmentState {
                    module: self.base.shader,
                    entry_point: Some(self.base.fragment_entry),
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &[Some(ColorTargetState {
                        format: self.base.fragment_format,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: self.topology,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
    }
}
