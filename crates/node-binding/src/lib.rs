#![deny(clippy::all)]

mod js_loader;
mod js_loader_types;

use feopack_binding::options::RawOptions;
use js_loader::create_js_loader_runner;
use js_loader_types::{JsLoaderContextInput, JsLoaderResultOutput};
use napi::bindgen_prelude::*;
use napi::{Error, Result};
use napi_derive::napi;
use std::sync::Arc;

use feopack_core::*;

#[napi]
pub struct Rspack {
  compiler: Box<Compiler>,
  js_runner: Option<Arc<FunctionRef<JsLoaderContextInput, JsLoaderResultOutput>>>,
}

#[napi]
impl Rspack {
  #[napi(constructor)]
  pub fn new(
    options: RawOptions,
    // 这里有一个 FFI 参数，从 js 那那边拿到 js loader runner 的句柄
    #[napi(
      ts_arg_type = "(ctx: JsLoaderContextInput) => JsLoaderResultOutput"
    )]
    js_runner: Option<Function<JsLoaderContextInput, JsLoaderResultOutput>>,
  ) -> Result<Self> {
    // 读 js 那边 注册的 loader
    let module_rules = options
      .module
      .map(|module| {
        module
          .rules
          .into_iter()
          .map(|rule| ModuleRuleOptions {
            test: rule.test,
            use_loaders: rule.use_loaders,
          })
          .collect::<Vec<_>>()
      })
      .unwrap_or_default();

    let compilation_options = CompilationOptions {
      mode: options.mode,
      entry: options.entry,
      context: options.context,
      output: Output {
        path: options.output.path,
        filename: options.output.filename,
      },
      module_rules,
    };

    let compiler = Compiler::new(compilation_options);

    let js_runner = js_runner
      .map(|runner_fn| runner_fn.create_ref().map(Arc::new))
      .transpose()?;

    Ok(Self {
      compiler: Box::new(compiler),
      js_runner,
    })
  }

  #[napi]
  pub fn build(&mut self, env: Env) -> Result<()> {
    if let Some(runner_ref) = self.js_runner.as_ref() {
      let runner = create_js_loader_runner(Arc::clone(runner_ref), env);
      self.compiler.set_js_loader_runner(Some(runner));
    }

    self
      .compiler
      .build_sync()
      .map_err(|e| Error::from_reason(e))
  }
}
