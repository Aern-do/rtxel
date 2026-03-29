use bevy_ecs::{
    resource::Resource,
    schedule::{IntoScheduleConfigs, ScheduleLabel},
    system::{Commands, Res, ResMut},
    world::World,
};
use rtxel_core::{Plugin, Startup, WorldExt};
use rtxel_gpu::{
    AsBindGroup, BasePipeline, Binding, Ctx, Float, Fragment, NonFiltering, TextureSampler,
    binding::Texture2D,
};
use wgpu::{
    BindGroup, BindGroupLayout, Color, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, Sampler, StoreOp,
    wgt::{SamplerDescriptor, TextureViewDescriptor},
};

use crate::{Frame, PipelineSet, RenderStartupSet, shared::SharedResources};

type PresentBindGroup = (
    Binding<0, Fragment, Texture2D<Float<false>>>,
    Binding<1, Fragment, TextureSampler<NonFiltering>>,
);

pub struct PresentPipelinePlugin<S> {
    pub schedule: S,
}

impl<S: ScheduleLabel> Plugin for PresentPipelinePlugin<S> {
    fn init(self, world: &mut World) {
        world
            .add_systems(Startup, init_pipeline.in_set(RenderStartupSet::Resources))
            .add_systems(
                self.schedule,
                (
                    rebuild
                        .in_set(PipelineSet::Extract)
                        .run_if(|shared: Res<SharedResources>| shared.is_dirty),
                    dispatch.in_set(PipelineSet::Dispatch),
                ),
            );
    }
}

const SHADER: &str = include_str!(env!("SHADER_present"));

#[derive(Debug, Clone, Resource)]
pub struct PresentPipeline {
    pub pipeline: RenderPipeline,

    pub sampler: Sampler,
    pub bg: BindGroup,
    pub bg_layout: BindGroupLayout,
}

impl PresentPipeline {
    pub fn new(shared: &SharedResources, ctx: &Ctx) -> Self {
        let shader = ctx.wgsl_shader(Some("Present Shadfer"), SHADER);

        let sampler = ctx.device.create_sampler(&SamplerDescriptor::default());

        let bg_layout = PresentBindGroup::layout(ctx);
        let bg = Self::create_bind_group(&bg_layout, shared, &sampler, ctx);

        let pipeline_layout =
            ctx.pipeline_layout(Some("Present Pipeline Layout"), &[Some(&bg_layout)]);
        let pipeline = ctx
            .render_pipeline(BasePipeline {
                layout: &pipeline_layout,
                shader: &shader,
                vertex_entry: "vs_main",
                fragment_entry: "fs_main",
                fragment_format: ctx.config.lock().expect("failed to lock config").format,
            })
            .label("Present Pipeline")
            .build();

        Self {
            pipeline,
            sampler,
            bg,
            bg_layout,
        }
    }

    pub fn create_bind_group(
        layout: &BindGroupLayout,
        shared: &SharedResources,
        sampler: &Sampler,
        ctx: &Ctx,
    ) -> BindGroup {
        let view = shared
            .out_texture
            .create_view(&TextureViewDescriptor::default());

        PresentBindGroup::bind_group(ctx, layout, (&view, sampler))
    }
}

fn init_pipeline(shared: Res<SharedResources>, ctx: Res<Ctx>, mut commands: Commands) {
    commands.insert_resource(PresentPipeline::new(&shared, &ctx));
}

fn rebuild(mut pipeline: ResMut<PresentPipeline>, shared: Res<SharedResources>, ctx: Res<Ctx>) {
    pipeline.bg =
        PresentPipeline::create_bind_group(&pipeline.bg_layout, &shared, &pipeline.sampler, &ctx)
}

fn dispatch(
    mut frame: ResMut<Frame>,
    pipeline: Res<PresentPipeline>,
    resources: Res<PresentPipeline>,
) {
    let Some(surface) = frame.surface() else {
        return;
    };

    let view = surface
        .texture
        .create_view(&TextureViewDescriptor::default());

    let encoder = frame.encoder_mut();
    {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Present Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            ..Default::default()
        });

        pass.set_pipeline(&pipeline.pipeline);
        pass.set_bind_group(0, &resources.bg, &[]);
        pass.draw(0..3, 0..1);
    }
}
