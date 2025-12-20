use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Output {
  pub path: String,
  pub filename: String,
}

// 对齐 Node-Binding 并做一些 Rust 侧的类型适配。（同时也避免循环依赖）
#[derive(Debug, Clone)]
pub struct CompilationOptions {
  pub mode: String,
  pub context: String,
  pub output: Output,
}

#[derive(Debug)]
pub struct Compilation {
  // id: String,
  pub options: Arc<CompilationOptions>,
}

impl Compilation {
  pub fn new(options: CompilationOptions) -> Self {
    println!("\n\nCompilation new: {:?}\n\n", options);
    Self {
      options: Arc::new(options),
    }
  }
}
