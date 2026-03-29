use std::{io::Result, path::Path};

use rtxel_slang::compile_slang;

fn main() {
    compile_shaders().expect("failed to compile shaders")
}

fn compile_shaders() -> Result<()> {
    let base_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");

    compile_slang(base_path.join("unpack.slang"), "unpack")?;
    compile_slang(base_path.join("draw.slang"), "draw")?;
    compile_slang(base_path.join("present.slang"), "present")
}
