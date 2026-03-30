use std::{
    env::{temp_dir, var},
    fs::{read_to_string, write},
    io::{Error, Result},
    path::Path,
    process::{Command, Output},
};

pub fn run_slangc(args: &[&str]) -> Result<Output> {
    let mut command = Command::new("slangc");
    command.args(args);

    command.output()
}

pub fn compile_slang_to_string(name: &str, path: &Path) -> Result<String> {
    let out_path = temp_dir().join(format!("{name}.wgsl"));

    let output = run_slangc(&[
        &path.as_os_str().to_string_lossy(),
        "-target",
        "wgsl",
        "-o",
        &out_path.to_string_lossy(),
    ])?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::other(format!("slangc failed: {stderr}")));
    }

    read_to_string(out_path)
}

pub fn compile_slang(path: impl AsRef<Path>, name: &str) -> Result<()> {
    let path = path.as_ref();
    let output = compile_slang_to_string(name, path)?;

    let out_dir = var("OUT_DIR").expect("OUT_DIR is missing");
    let dest = Path::new(&out_dir).join(format!("{name}.wgsl"));
    write(&dest, &output)?;

    println!("cargo::rerun-if-changed={}", path.display());
    println!("cargo::rustc-env=SHADER_{name}={}", dest.display());

    Ok(())
}

pub fn mark_as_dep(path: impl AsRef<Path>) {
    let path = path.as_ref();
    println!("cargo::rerun-if-changed={}", path.display());
}
