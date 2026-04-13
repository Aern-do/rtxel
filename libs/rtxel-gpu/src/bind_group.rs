use std::marker::PhantomData;

use paste::paste;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, ShaderStages,
};

use crate::Ctx;

pub trait Bindable {
    type Resource: 'static;

    fn binding_type() -> BindingType;
    fn binding_resource(resource: &Self::Resource) -> BindingResource<'_>;
}

pub trait Visibility {
    fn visibility() -> ShaderStages;
}

pub struct Compute;

impl Visibility for Compute {
    fn visibility() -> ShaderStages {
        ShaderStages::COMPUTE
    }
}

pub struct Vertex;

impl Visibility for Vertex {
    fn visibility() -> ShaderStages {
        ShaderStages::VERTEX
    }
}

pub struct Fragment;

impl Visibility for Fragment {
    fn visibility() -> ShaderStages {
        ShaderStages::FRAGMENT
    }
}

pub struct VertexFragment;

impl Visibility for VertexFragment {
    fn visibility() -> ShaderStages {
        ShaderStages::VERTEX_FRAGMENT
    }
}

pub struct Binding<const IDX: usize, V, B>(PhantomData<(V, B)>);

pub type ComputeBinding<const IDX: usize, B> = Binding<IDX, Compute, B>;

pub trait AsBindGroup {
    type Resources<'res>;

    fn layout(ctx: &Ctx) -> BindGroupLayout;

    fn bind_group<'res>(
        ctx: &Ctx,
        layout: &BindGroupLayout,
        resources: Self::Resources<'res>,
    ) -> BindGroup;
}

impl<const IDX: usize, V: Visibility, B: Bindable> AsBindGroup for Binding<IDX, V, B> {
    type Resources<'res> = &'res B::Resource;

    fn layout(ctx: &Ctx) -> BindGroupLayout {
        ctx.device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[BindGroupLayoutEntry {
                    binding: IDX as u32,
                    visibility: V::visibility(),
                    ty: B::binding_type(),
                    count: None,
                }],
            })
    }

    fn bind_group<'res>(
        ctx: &Ctx,
        layout: &BindGroupLayout,
        resources: Self::Resources<'res>,
    ) -> BindGroup {
        ctx.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries: &[BindGroupEntry {
                binding: IDX as u32,
                resource: B::binding_resource(resources),
            }],
        })
    }
}

macro_rules! impl_as_bind_group {
    ($($($generic:ident)*);*) => {paste!{$(
        #[allow(non_snake_case)]
        impl<
            $(
                const [<IDX $generic>]: usize,
                [<V $generic>]: Visibility,
                [<B $generic>]: Bindable,
            )*
        > AsBindGroup for (
            $(Binding<[<IDX $generic>], [<V $generic>], [<B $generic>]>,)*
        ) {
            type Resources<'res> = (
                $(&'res [<B $generic>]::Resource,)*
            );

            fn layout(ctx: &Ctx) -> BindGroupLayout {
                ctx.device.
                    create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[
                            $(BindGroupLayoutEntry {
                                binding: [<IDX $generic>] as u32,
                                visibility: [<V $generic>]::visibility(),
                                ty: [<B $generic>]::binding_type(),
                                count: None,
                            },)*]
                        })
            }

            fn bind_group<'res>(
                ctx: &Ctx,
                layout: &BindGroupLayout,
                resources: Self::Resources<'res>,
            ) -> BindGroup {
                let ($([<r $generic>],)*) = resources;


                ctx.device.create_bind_group(&BindGroupDescriptor {
                    label: None,
                    layout,
                    entries: &[
                        $(BindGroupEntry {
                            binding: [<IDX $generic>] as u32,
                            resource: [<B $generic>]::binding_resource(
                                [<r $generic>],
                            ),
                        },)*
                    ],
                })
            }
        }
    )*}};
}

impl_as_bind_group! {
    A;
    A B;
    A B C;
    A B C D;
    A B C D E;
    A B C D E F;
    A B C D E F G;

}
