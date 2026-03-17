use rtxel_core::{WorldExt, start::start};
use rtxel_game::GamePlugin;
use rtxel_render::RenderPlugin;

fn main() {
    start(
        |world| {
            world.add_plugin(RenderPlugin);
            world.add_plugin(GamePlugin);
        },
        Default::default(),
    );
}
