use std::{io::Result, path::Path};

use rtxel_slang::{compile_slang, mark_as_dep};

fn main() {
    compile_shaders().expect("failed to compile shaders")
}

fn compile_shaders() -> Result<()> {
    let base_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");

    compile_slang(base_path.join("unpack.slang"), "unpack")?;
    compile_slang(base_path.join("draw.slang"), "draw")?;
    compile_slang(base_path.join("present.slang"), "present")?;

    // TODO: find deps automaticly via slang reflection
    mark_as_dep(base_path.join("world.slang"));
    mark_as_dep(base_path.join("traverse.slang"));
    mark_as_dep(base_path.join("core").join("dda.slang"));
    mark_as_dep(base_path.join("core").join("ray.slang"));
    mark_as_dep(base_path.join("core").join("volume.slang"));


    Ok(())
}
