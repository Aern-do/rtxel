use std::{io::Result, path::Path};

use rtxel_slang::{compile_slang_dir, mark_dir_as_dep};

fn main() {
    match compile_shaders() {
        Ok(..) => {}
        Err(err) => {
            println!("{err}");
            panic!("{err:?}");
        }
    }
}

fn compile_shaders() -> Result<()> {
    let base_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");

    mark_dir_as_dep(base_path.join("core"))?;
    compile_slang_dir(base_path.join("passes"), &[&base_path])?;

    Ok(())
}
