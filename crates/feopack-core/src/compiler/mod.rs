pub mod compilation;
mod emit;
mod lifecycle;

use crate::loader::JsLoaderRunner;
use compilation::{Compilation, CompilationOptions};
use std::result::Result;

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
