pub mod compilation;

use crate::utils::fast_set;
use compilation::{Compilation, CompilationOptions};
use std::path::Path;
use std::result::Result;
use tokio::fs;

pub struct Compiler {
  compilation: Compilation,
  options: CompilationOptions,
}

impl Compiler {
  pub fn new(options: CompilationOptions) -> Self {
    Self {
      compilation: Compilation::new(options.clone()),
      options,
    }
  }

  pub async fn build(&mut self) -> Result<(), String> {
    let compilation = Compilation::new(self.options.clone());
    fast_set(&mut self.compilation, compilation);
    self.compile().await?;
    // TODO: compile_done()
    Ok(())
  }

  pub async fn emit_assets(&mut self) -> Result<(), String> {
    for asset in &self.compilation.assets {
      let output_dir = Path::new(&self.options.output.path);
      let output_file = output_dir.join(&asset.filename);

      if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)
          .await
          .map_err(|e| format!("创建目录失败 {:?}: {}", parent, e))?;
      }

      fs::write(&output_file, asset.source.as_bytes())
        .await
        .map_err(|e| format!("写入失败 {:?}: {}", output_file, e))?;
    }

    Ok(())
  }

  pub async fn compile(&mut self) -> Result<(), String> {
    self.compilation.make().await?;
    self.compilation.seal().await?;
    self.emit_assets().await?;

    Ok(())
  }
}
