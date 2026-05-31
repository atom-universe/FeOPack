mod code_generation;
mod make;
mod resolve;
mod seal;

use crate::loader::text_loader::text_loader;
use crate::loader::{LoaderRegistry, LoaderRule};
use crate::module_graph::ModuleGraph;
use std::collections::HashMap;
use std::path::PathBuf;

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

#[derive(Debug, Clone)]
pub struct Chunk {
  pub id: String,
  pub module_ids: Vec<String>,
}

#[derive(Debug, Default)]
pub struct ChunkGraph {
  pub chunks: Vec<Chunk>,
}

#[derive(Debug, Clone)]
pub struct GeneratedAsset {
  pub filename: String,
  pub source: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CodegenModule {
  // code_generation 阶段消费的临时结构：
  // 到这里已经不是“源码文件”，而是即将塞进 chunk runtime 的模块函数体。
  pub(crate) id: String,
  pub(crate) source: String,
}

#[derive(Debug, Clone)]
pub(crate) enum ResolvedPath {
  File(PathBuf),
  External(String),
}

#[derive(Debug)]
pub struct Compilation {
  pub options: CompilationOptions,
  pub module_graph: ModuleGraph,
  pub chunk_graph: ChunkGraph,
  pub assets: Vec<GeneratedAsset>,
  // make 阶段的 build result 先简单放这里。
  // 真实 bundler 通常会把这类信息挂在 Module/BuildResult 上；这里先用 map 保持 MVP 清晰。
  pub(crate) module_sources: HashMap<String, String>,
  pub(crate) loader_registry: LoaderRegistry,
}

impl Compilation {
  pub fn new(options: CompilationOptions) -> Self {
    println!("\n\nCompilation new: {:?}\n\n", options);
    let mut loader_registry = LoaderRegistry::new();
    loader_registry.register_loader("text-loader".to_string(), text_loader);
    loader_registry.add_rule(LoaderRule {
      test: ".txt".to_string(),
      used_loaders: vec!["text-loader".to_string()],
    });

    Self {
      options: options.clone(),
      module_graph: ModuleGraph::new(),
      chunk_graph: ChunkGraph::default(),
      assets: Vec::new(),
      module_sources: HashMap::new(),
      loader_registry,
    }
  }

  pub async fn make(&mut self) -> Result<(), String> {
    self.build_module_graph().await
  }

  pub async fn seal(&mut self) -> Result<(), String> {
    // module graph 到这里以后先视为稳定，seal 负责把它组织成 chunk，并继续生成 assets。
    println!(
      "\n[rust seal 阶段] module graph -> chunk graph:\n {:?}\n",
      self.module_graph.partials
    );
    self.create_chunk_graph().await;
    self.code_generation().await?;
    Ok(())
  }
}
