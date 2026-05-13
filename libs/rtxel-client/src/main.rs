pub mod engine;
pub mod start;
pub mod world;

pub use engine::{Engine, start};
pub use start::{Event, Start};

fn main() {
    env_logger::init();
    start();
}
