use bevy_ecs::resource::Resource;
use encase::{ShaderType, UniformBuffer};
use glam::Vec3;
use rtxel_gpu::{BaseComputePipeline, BasePipeline as BaseRenderPipeline, Ctx, binding, layout};
use wgpu::{
    BindGroup, Buffer, BufferUsages, ComputePipeline, RenderPipeline, Texture, TextureFormat,
    TextureUsages,
    wgt::{SamplerDescriptor, TextureViewDescriptor},
};

const COMPUTE_SRC: &str = include_str!(env!("SHADER_compute"));
const DRAW_SRC: &str = include_str!(env!("SHADER_draw"));

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct CameraUniform {
    pub origin: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    pub fov: f32,
    pub aspect: f32,
}

#[derive(Resource)]
pub struct Pipeline {
    pub out_texture: Texture,
    pub compute: Compute,
    pub draw: Draw,
}

impl Pipeline {
    pub fn new(ctx: &Ctx) -> Self {
        let config = ctx.config.lock().unwrap();
        let width = config.width;
        let height = config.height;

        drop(config);

        let out_texture = ctx.texture(
            Some("storage_texture"),
            width,
            height,
            TextureFormat::Rgba8Unorm,
            TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        );

        let compute = Compute::new(ctx, &out_texture);
        let draw = Draw::new(ctx, &out_texture);

        Self {
            out_texture,
            compute,
            draw,
        }
    }
}

#[derive(Debug)]
pub struct Compute {
    pub pipeline: ComputePipeline,
    pub bg: BindGroup,
    pub camera_buffer: Buffer,
}

impl Compute {
    pub fn new(ctx: &Ctx, out_texture: &Texture) -> Self {
        let shader = ctx.wgsl_shader(Some("Compute Shader"), COMPUTE_SRC);
        let out_view = out_texture.create_view(&TextureViewDescriptor::default());

        let camera_buffer = ctx.buffer(
            Some("camera_uniform_buffer"),
            <CameraUniform as ShaderType>::min_size().get(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let bg_layout = ctx.bind_group_layout(
            Some("Compute Bind Group Layout"),
            &[
                layout::compute(0, layout::w_storage_texture(out_texture)),
                layout::compute(
                    1,
                    layout::uniform_buffer(Some(<CameraUniform as ShaderType>::min_size())),
                ),
            ],
        );

        let bg = ctx.bind_group(
            Some("Compute Bind Group"),
            &bg_layout,
            &[
                binding::entry(0, binding::view(&out_view)),
                binding::entry(1, binding::buffer(&camera_buffer)),
            ],
        );

        let pipeline_layout = ctx.pipeline_layout(Some("Compute Pipeline Layout"), &[&bg_layout]);

        let pipeline = ctx
            .compute_pipeline(BaseComputePipeline {
                layout: &pipeline_layout,
                shader: &shader,
                entry: "main",
            })
            .label("Compute Pipeline")
            .build();

        Self {
            pipeline,
            bg,
            camera_buffer,
        }
    }
}

#[derive(Debug)]
pub struct Draw {
    pub pipeline: RenderPipeline,
    pub bg: BindGroup,
}

impl Draw {
    pub fn new(ctx: &Ctx, out_texture: &Texture) -> Self {
        let shader = ctx.wgsl_shader(Some("Draw Shader"), DRAW_SRC);

        let out_view = out_texture.create_view(&TextureViewDescriptor::default());
        let sampler = ctx.device.create_sampler(&SamplerDescriptor::default());

        let bg_layout = ctx.bind_group_layout(
            Some("Draw Bind Group Layout"),
            &[
                layout::fragment(0, layout::texture_float()),
                layout::fragment(1, layout::sampler_filtering()),
            ],
        );

        let bind_group = ctx.bind_group(
            Some("Draw Bind Group"),
            &bg_layout,
            &[
                binding::entry(0, binding::view(&out_view)),
                binding::entry(1, binding::sampler(&sampler)),
            ],
        );

        let pipeline_layout = ctx.pipeline_layout(Some("Draw Pipeline Layout"), &[&bg_layout]);

        let pipeline = ctx
            .render_pipeline(BaseRenderPipeline {
                layout: &pipeline_layout,
                shader: &shader,
                vertex_entry: "vs",
                fragment_entry: "fs",
                fragment_format: ctx.config.lock().expect("failed to lock config").format,
            })
            .label("Draw Pipeline")
            .build();

        Self {
            pipeline,
            bg: bind_group,
        }
    }
}
