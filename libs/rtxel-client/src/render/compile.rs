use std::{
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use log::info;
use rtxel_gpu::Ctx;
use wgpu::{ShaderModule, ShaderModuleDescriptor, ShaderSource};

/// Compiles Slang shader to WGSL, requires slangc to be installed
// TODO: install slangc instead of relaying on user having it installed already
pub fn compile(path: &Path, search_path: &Path) -> Result<String> {
    info!("compiling shader: {}", path.display());

    let output = Command::new("slangc")
        .arg(path)
        .arg("-I")
        .arg(search_path)
        .arg("-target")
        .arg("wgsl")
        .output()?;

    if !output.status.success() {
        return Err(Error::other(format!(
            "slangc failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        )));
    }

    String::from_utf8(output.stdout).map_err(|err| Error::new(ErrorKind::InvalidData, err))
}

/// Compiles Slang shader to WGSL and creates shader module, requires slangc to be installed
pub fn compile_into_shader_module(
    path: &Path,
    search_path: &Path,
    ctx: &Ctx,
) -> Result<ShaderModule> {
    let shader = compile(path, search_path)?;

    Ok(ctx.device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(shader.into()),
    }))
}

/// Compiles shader in given base path using `slangc`
#[derive(Debug, Clone)]
pub struct Compiler {
    ctx: Arc<Ctx>,
    base_path: PathBuf,
}

impl Compiler {
    pub fn new(ctx: Arc<Ctx>, base_path: PathBuf) -> Self {
        Self { ctx, base_path }
    }

    pub fn compile(&self, path: impl AsRef<Path>) -> Result<ShaderModule> {
        compile_into_shader_module(&self.base_path.join(path), &self.base_path, &self.ctx)
    }
}
