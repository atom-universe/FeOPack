use crate::module_graph::ModuleGraph;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Output {
  pub path: String,
  pub filename: String,
}

// 对齐 Node-Binding 并做一些 Rust 侧的类型适配。（同时也避免循环依赖）
#[derive(Debug, Clone)]
pub struct CompilationOptions {
  pub entry: String,
  pub mode: String,
  pub context: String,
  pub output: Output,
}

#[derive(Debug)]
pub struct Compilation {
  // id: String,
  // TODO: 后面要做多线程的话得加 Arc
  pub options: CompilationOptions,
}

impl Compilation {
  pub fn new(options: CompilationOptions) -> Self {
    println!("\n\nCompilation new: {:?}\n\n", options);
    Self {
      options: options.clone(),
    }
  }

  // 产生 module graph (module + deps)
  pub async fn make(&self) -> Result<(), String> {
    let entry = self.options.entry.clone();
    println!("\n\n[rust] make 阶段: {:?}\n\n", entry);
    Ok(())
  }

  // make 完成后的检查阶段，但是这里先不实现了
  pub fn finish(&self) {
    todo!()
  }

  // module graph -> chunk graph
  pub fn seal(&self) {
    todo!()
  }
}
