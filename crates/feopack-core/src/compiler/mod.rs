pub mod compilation;
mod emit;
mod hooks;
mod lifecycle;
mod normal_module_factory;
mod plugin;

use crate::loader::JsLoaderRunner;
use compilation::{Compilation, CompilationOptions};
use hooks::CompilerHooks;
pub use plugin::{apply_builtin_plugin, Plugin};
use std::result::Result;

pub struct Compiler {
  compilation: Compilation,
  options: CompilationOptions,
  js_loader_runner: Option<JsLoaderRunner>,
  hooks: CompilerHooks,
}

impl Compiler {
  pub fn new(options: CompilationOptions) -> Self {
    Self {
      compilation: Compilation::new(options.clone(), None),
      options,
      js_loader_runner: None,
      hooks: CompilerHooks::default(),
    }
  }

  pub fn set_js_loader_runner(&mut self, runner: Option<JsLoaderRunner>) {
    self.js_loader_runner = runner;
  }

  #[allow(dead_code)]
  pub(crate) fn hooks_mut(&mut self) -> &mut CompilerHooks {
    &mut self.hooks
  }

  pub async fn build(&mut self) -> Result<(), String> {
    self.initialize()?;
    self.before_run()?;
    self.run_lifecycle()?;
    self.compilation = Compilation::new(self.options.clone(), self.js_loader_runner.clone());
    self.compile().await?;
    self.done()?;
    Ok(())
  }

  pub async fn compile(&mut self) -> Result<(), String> {
    self.before_compile()?;
    self.compile_lifecycle()?;
    self.compilation.make().await?;
    self.compilation.seal().await?;
    self.after_compile()?;
    if !self.should_emit()? {
      println!("[rust compiler lifecycle] should_emit returned false, skip emit");
      return Ok(());
    }
    self.emit_assets().await?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::compilation::{CompilationOptions, Output};
  use super::Compiler;
  use crate::compiler::compilation::GeneratedAsset;
  use std::path::Path;
  use std::sync::{Arc, Mutex};

  fn test_compiler() -> Compiler {
    Compiler::new(CompilationOptions {
      entry: "src/index.js".to_string(),
      mode: "development".to_string(),
      context: "/tmp/feopack-test".to_string(),
      output: Output {
        path: "/tmp/feopack-test/dist".to_string(),
        filename: "main.js".to_string(),
      },
      module_rules: Vec::new(),
    })
  }

  #[test]
  fn lifecycle_calls_registered_compiler_hook() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut compiler = test_compiler();

    let before_run_calls = Arc::clone(&calls);
    compiler
      .hooks_mut()
      .before_run
      .tap("test-before-run", move |_| {
        before_run_calls
          .lock()
          .expect("lock calls")
          .push("before_run");
        Ok(())
      });

    compiler.before_run().expect("before_run should pass");

    assert_eq!(*calls.lock().expect("lock calls"), vec!["before_run"]);
  }

  #[test]
  fn asset_emitted_hook_receives_context() {
    let seen = Arc::new(Mutex::new(None));
    let mut compiler = test_compiler();

    let seen_context = Arc::clone(&seen);
    compiler
      .hooks_mut()
      .asset_emitted
      .tap("test-asset-emitted", move |ctx| {
        *seen_context.lock().expect("lock seen") =
          Some((ctx.filename.clone(), ctx.target_path.clone()));
        Ok(())
      });

    compiler
      .asset_emitted(
        &GeneratedAsset {
          filename: "main.js".to_string(),
          source: String::new(),
        },
        Path::new("/tmp/feopack-test/dist/main.js"),
      )
      .expect("asset_emitted should pass");

    assert_eq!(
      *seen.lock().expect("lock seen"),
      Some((
        "main.js".to_string(),
        Path::new("/tmp/feopack-test/dist/main.js").to_path_buf()
      ))
    );
  }

  #[test]
  fn should_emit_bail_hook_can_skip_emit() {
    let mut compiler = test_compiler();

    compiler
      .hooks_mut()
      .should_emit
      .tap("skip-emit", |_| Ok(Some(false)));

    assert!(!compiler.should_emit().expect("should_emit should pass"));
  }
}
