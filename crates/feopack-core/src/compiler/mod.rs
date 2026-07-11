pub mod compilation;
mod lifecycle;

use crate::loader::JsLoaderRunner;
use compilation::{Compilation, CompilationOptions};
use std::path::Path;
use std::result::Result;
use tokio::fs;

pub struct Compiler {
  compilation: Compilation,
  options: CompilationOptions,
  js_loader_runner: Option<JsLoaderRunner>,
}

impl Compiler {
  pub fn new(options: CompilationOptions) -> Self {
    Self {
      compilation: Compilation::new(options.clone(), None),
      options,
      js_loader_runner: None,
    }
  }

  pub fn set_js_loader_runner(&mut self, runner: Option<JsLoaderRunner>) {
    self.js_loader_runner = runner;
  }

  pub async fn build(&mut self) -> Result<(), String> {
    self.initialize();
    self.before_run();
    self.run_lifecycle();
    self.compilation = Compilation::new(self.options.clone(), self.js_loader_runner.clone());
    self.compile().await?;
    self.done();
    Ok(())
  }

  pub async fn emit_assets(&mut self) -> Result<(), String> {
    self.emit();
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
      self.asset_emitted(asset, &output_file);
    }

    self.after_emit();
    Ok(())
  }

  pub async fn compile(&mut self) -> Result<(), String> {
    self.before_compile();
    self.compile_lifecycle();
    self.compilation.make().await?;
    self.compilation.seal().await?;
    self.after_compile();
    self.emit_assets().await?;
    Ok(())
  }
}
