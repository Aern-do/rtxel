pub mod camera;
pub mod engine;
pub mod keyboard;
pub mod render;
pub mod start;
pub mod world;

pub use camera::Camera;
pub use engine::{Engine, start};
pub use keyboard::Keyboard;
pub use start::{Event, Start};

fn main() {
    env_logger::init();
    start();
}
