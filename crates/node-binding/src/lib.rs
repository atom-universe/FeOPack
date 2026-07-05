#![deny(clippy::all)]

mod js_loader;
mod js_loader_types;

use feopack_binding::options::RawOptions;
use js_loader::{create_js_loader_runner, JsLoaderRunnerTsFn};
use js_loader_types::{JsLoaderContextInput, JsLoaderResultOutput};
use napi::bindgen_prelude::*;
use napi::{Error, Result};
use napi_derive::napi;
use std::sync::Arc;

use feopack_core::*;

#[napi]
pub struct Rspack {
  compiler: Box<Compiler>,
  js_runner: Option<Arc<JsLoaderRunnerTsFn>>,
}

#[napi]
impl Rspack {
  #[napi(constructor)]
  pub fn new(
    options: RawOptions,
    // 这里有一个 FFI 参数，从 js 那那边拿到 js loader runner 的句柄
    #[napi(
      ts_arg_type = "(ctx: JsLoaderContextInput) => Promise<JsLoaderResultOutput>"
    )]
    js_runner: Option<Function<JsLoaderContextInput, Promise<JsLoaderResultOutput>>>,
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
      .map(|runner_fn| -> Result<Arc<JsLoaderRunnerTsFn>> {
        let ts_fn: JsLoaderRunnerTsFn = runner_fn
          .build_threadsafe_function::<JsLoaderContextInput>()
          .max_queue_size::<0>()
          .build()?;
        Ok(Arc::new(ts_fn))
      })
      .transpose()?;

    Ok(Self {
      compiler: Box::new(compiler),
      js_runner,
    })
  }

  #[napi]
  pub async unsafe fn build(&mut self) -> Result<()> {
    if let Some(runner_ref) = self.js_runner.as_ref() {
      let runner = create_js_loader_runner(Arc::clone(runner_ref));
      self.compiler.set_js_loader_runner(Some(runner));
    }

    self
      .compiler
      .build()
      .await
      .map_err(|e| Error::from_reason(e))
  }
}
