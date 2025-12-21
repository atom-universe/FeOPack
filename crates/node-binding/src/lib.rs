#![deny(clippy::all)]

use feopack_binding::options::RawOptions;
use napi::{Error, Result};
use napi_derive::napi;

use feopack_core::*;

#[napi]
pub struct Rspack {
  compiler: Box<Compiler>,
}

#[napi]
impl Rspack {
  #[napi(constructor)]
  pub fn new(options: RawOptions) -> Result<Self> {
    // 从 RawOptions 转换为 CompilationOptions
    let compilation_options = CompilationOptions {
      mode: options.mode,
      entry: options.entry,
      context: options.context,
      output: Output {
        path: options.output.path,
        filename: options.output.filename,
      },
    };

    let compiler = Compiler::new(compilation_options);
    Ok(Self {
      compiler: Box::new(compiler),
    })
  }

  #[napi]
  pub async unsafe fn build(&mut self) -> Result<()> {
    self
      .compiler
      .build()
      .await
      .map_err(|e| Error::from_reason(e.to_string()))
  }
}
