use std::num::NonZero;

use wgpu::{BindGroup, BindGroupLayout, BindingResource, BindingType};

use crate::Ctx;

/// Implicit variant of [BindGroupLayoutEntry](wgpu::BindGroupLayoutEntry) lacking visibility and index.
/// Visibility and index are specified later when creating bind group using `group!` macro.
pub struct ImplicitBindGroupLayoutEntry {
    pub ty: BindingType,
    pub count: Option<NonZero<u32>>,
}

/// Describes a binding in a bind group
pub trait Bind {
    /// Resource of this binding
    type Resource;

    /// Returns bind group layout of this binding
    fn layout() -> ImplicitBindGroupLayoutEntry;

    /// Returns [BindGroupLayout] of this binding
    fn resource<'res>(resource: &'res Self::Resource) -> BindingResource<'res>;
}

/// Describes a bind group
pub trait AsBindGroup {
    /// Create a bind group
    fn group(&self, ctx: &Ctx) -> BindGroup;

    // Create a bind group layout
    fn layout(ctx: &Ctx) -> BindGroupLayout;
}

/// Macro to produce an binding group.
///
/// The input is struct with fields explicitly describing shader visibility and optionally binding index.
///
/// # Example
/// ```rust
/// group!(pub struct Group {
///     // Binding 0: read-only storage buffer, visible in compute, vertex and fragment
///     pub [compute, vertex, fragment] foo: StorageBuffer<true>,
///     // Binding 3: writable storage buffer, visible in vertex only
///     pub [vertex] @ 3 bar: StorageBuffer<false>,
///     // Binding 4: storage texture, visible in compute only
///     pub [compute] output: D2StorageTexture<ReadWrite, Rgba8Unorm>
/// })
/// ```
#[macro_export]
macro_rules! group {
    ($vis:vis struct $name:ident {
        $($bind_vis:vis [$($bind_stage:ident),+] $(@$bind_idx:literal)? $bind_name:ident: $bind_ty:ty),* $(,)?
    }) => {
        $vis struct $name<'res> {
            $($bind_vis $bind_name: &'res <$bind_ty as $crate::Bind>::Resource),*
        }

        impl<'res> $crate::AsBindGroup for $name<'res> {
            #[allow(unused)]
            fn group(&self, ctx: &$crate::Ctx) -> wgpu::BindGroup {
                let mut binding = 0;
                ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &Self::layout(ctx),
                    entries: &[$({
                        binding = $crate::group!(@resolve_binding binding, $($bind_idx)?);
                        let entry = wgpu::BindGroupEntry {
                            binding,
                            resource: <$bind_ty as $crate::Bind>::resource(self.$bind_name),
                        };
                        binding += 1;
                        entry
                    }),*]
                })

            }

            #[allow(unused)]
            fn layout(ctx: &$crate::Ctx) -> wgpu::BindGroupLayout {
                let mut binding = 0;
                ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[$({
                        binding = $crate::group!(@resolve_binding binding, $($bind_idx)?);
                        let layout = <$bind_ty as $crate::Bind>::layout();
                        let entry = wgpu::BindGroupLayoutEntry {
                            binding,
                            visibility: $($crate::group!(@stage $bind_stage))|*,
                            ty: layout.ty,
                            count: layout.count,
                        };
                        binding += 1;
                        entry
                    }),*]
                })
            }
        }
    };

    (@resolve_binding $binding:ident, $bind_idx:literal) => { $bind_idx };
    (@resolve_binding $binding:ident,) => { $binding };

    (@stage vertex) => {
        wgpu::ShaderStages::VERTEX
    };
    (@stage fragment) => {
        wgpu::ShaderStages::FRAGMENT
    };
    (@stage compute) => {
        wgpu::ShaderStages::COMPUTE
    };
}
