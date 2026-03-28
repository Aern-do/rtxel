use rtxel_core::{WorldExt, start::start};
use rtxel_game::GamePlugin;

fn main() {
    env_logger::init();

    start(
        |world| {
            world.add_plugin(GamePlugin);
        },
        Default::default(),
    );
}
