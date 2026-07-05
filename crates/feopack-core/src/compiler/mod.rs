pub mod compilation;

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
    self.compilation = Compilation::new(self.options.clone(), self.js_loader_runner.clone());
    self.compile().await?;
    // TODO: compile_done()
    Ok(())
  }

  // napi 对外是同步 build()；内部仍用 async 管线，在这里阻塞跑完（见下方各行注释）
  pub fn build_sync(&mut self) -> Result<(), String> {
    // 一个我不知道的知识：
    // tokio 并不是我曾以为的那种多线程库，而是一个 async runtime 库
    // tokio 其实有两种模式，一个是多线程的，一个是单线程的
    // 我们现在用的就是单线程的模式，走的就是事件循环的那种
    let rt = tokio::runtime::Builder::new_current_thread() 
    // 单线程 runtime：async 任务不跳到别的线程，方便在同一线程回调 JS loader
      .enable_all() // 打开 I/O 驱动；compile() 里的 tokio::fs 依赖它，否则会 “no reactor running”
      .build()
      .map_err(|e| format!("创建 tokio runtime 失败: {e}"))?;

    // 阻塞直到 build().await 整条链路结束，Node 侧 rspack.build() 才会返回
    rt.block_on(self.build())
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
