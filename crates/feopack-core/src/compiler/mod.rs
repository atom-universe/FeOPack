pub mod compilation;

use crate::utils::fast_set;
use compilation::{Compilation, CompilationOptions};
use std::result::Result;

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
    Ok(())
  }
}
