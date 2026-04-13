use std::{
    env::{temp_dir, var},
    fs::{read_dir, read_to_string, write},
    io::{Error, Result},
    path::Path,
    process::{Command, Output},
};

pub fn run_slangc(args: &[&str]) -> Result<Output> {
    let mut command = Command::new("slangc");
    command.args(args);

    command.output()
}

pub fn compile_slang_to_string(name: &str, path: &Path, search_paths: &[&Path]) -> Result<String> {
    let out_path = temp_dir().join(format!("{name}.wgsl"));

    let mut args = vec![
        path.as_os_str().to_string_lossy().into_owned(),
        "-target".into(),
        "wgsl".into(),
        "-o".into(),
        out_path.to_string_lossy().into_owned(),
    ];

    for search_path in search_paths {
        args.push("-I".into());
        args.push(search_path.to_string_lossy().into_owned());
    }

    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = run_slangc(&arg_refs)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::other(format!("slangc failed: {stderr}")));
    }

    read_to_string(out_path)
}

pub fn compile_slang(path: impl AsRef<Path>, name: &str, search_paths: &[&Path]) -> Result<()> {
    let path = path.as_ref();
    let output = compile_slang_to_string(name, path, search_paths)?;

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

pub fn mark_dir_as_dep(dir: impl AsRef<Path>) -> Result<()> {
    let dir = dir.as_ref();
    mark_as_dep(dir);

    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            mark_dir_as_dep(&path)?;
        } else {
            mark_as_dep(&path);
        }
    }

    Ok(())
}

pub fn compile_slang_dir(dir: impl AsRef<Path>, search_paths: &[&Path]) -> Result<()> {
    let dir = dir.as_ref();
    mark_dir_as_dep(dir)?;

    compile_slang_dir_recursive(dir, search_paths)
}

fn compile_slang_dir_recursive(dir: &Path, search_paths: &[&Path]) -> Result<()> {
    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            compile_slang_dir_recursive(&path, search_paths)?;
        } else {
            let name = path
                .file_stem()
                .ok_or_else(|| Error::other("file has no stem"))?
                .to_string_lossy();

            compile_slang(&path, &name, search_paths)?;
        }
    }

    Ok(())
}
