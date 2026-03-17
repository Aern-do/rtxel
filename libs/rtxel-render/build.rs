use std::path::Path;

use rtxel_slang::compile_slang;

fn main() {
    let base_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");

    compile_slang(base_path.join("compute.slang"), "compute").expect("failed to compile shader");
    compile_slang(base_path.join("draw.slang"), "draw").expect("failed to compile shader");
}
