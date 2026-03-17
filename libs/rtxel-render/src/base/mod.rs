pub mod extract;
pub mod pipeline;
pub mod render;
pub use extract::*;
pub use pipeline::*;
pub use render::*;

use crate::Ctx;
use bevy_ecs::system::{Commands, Res};

pub fn init(mut commands: Commands, ctx: Res<Ctx>) {
    let pipeline = Pipeline::new(&ctx);
    commands.insert_resource(pipeline);
}
