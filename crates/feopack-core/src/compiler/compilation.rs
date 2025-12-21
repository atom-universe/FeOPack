use crate::module_graph::{Module, ModuleGraph};
use crate::swc_compiler::SwcCompiler;
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use swc_ecma_ast::{ModuleDecl, ModuleItem, Program};
use tokio::fs::read_to_string;

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

#[derive(Debug)]
pub struct Compilation {
  pub options: CompilationOptions,
  pub module_graph: ModuleGraph,
  pub chunk_graph: ChunkGraph,
}

impl Compilation {
  pub fn new(options: CompilationOptions) -> Self {
    println!("\n\nCompilation new: {:?}\n\n", options);
    Self {
      options: options.clone(),
      module_graph: ModuleGraph::new(),
      chunk_graph: ChunkGraph::default(),
    }
  }

  // 解析依赖路径，主要是处理 `/basic/./app.js` 这种带相对路径的情况
  fn resolve_dep_path(dep: &str, module_dir: &Path, context: &Path) -> Result<PathBuf, String> {
    // 如果依赖路径以 . 或 .. 开头，
    // Rust 的 Path::join 会自动处理 . 和 .. 的规范化
    let mut dep_path = if dep.starts_with('.') {
      module_dir.join(dep)
    } else {
      // 对于非相对路径（如 npm 包），使用 context
      context.join(dep)
    };

    // 检查文件是否存在，如果不存在且没有扩展名，尝试添加 .js
    if !dep_path.exists() && dep_path.extension().is_none() {
      let dep_path_with_js = dep_path.with_extension("js");
      if dep_path_with_js.exists() {
        dep_path = dep_path_with_js;
      }
    }

    Ok(dep_path)
  }

  // 产生 module graph (module + deps)
  /*
   * 这里 rspack 实际情况是把数据结构转为 EntryDependency, 然后通过 ModuleFactory 创建 Module
   * 另外还会开一个 Task Loop 做并行调度, 这里就用 BFS 大致模拟
   */
  pub async fn make(&mut self) -> Result<(), String> {
    let context = Path::new(&self.options.context);
    let entry_path = context.join(&self.options.entry);

    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();

    queue.push_back(entry_path);

    while let Some(module_path) = queue.pop_front() {
      if visited.contains(&module_path) {
        continue;
      }
      visited.insert(module_path.clone());

      let source = read_to_string(&module_path)
        .await
        .map_err(|e| format!("读取文件失败 {:?}: {}", module_path, e))?;

      let compiler = SwcCompiler::new();
      let ast = compiler.parse_js(module_path.clone(), source)?;

      let mut dependencies = Vec::new();

      if let Program::Module(module) = ast {
        // 获取当前模块的父目录，用于解析相对路径
        let module_dir = module_path
          .parent()
          .ok_or_else(|| format!("无法获取模块目录: {:?}", module_path))?;

        for item in module.body {
          if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item {
            if let Some(dep) = import.src.value.as_str() {
              let dep = dep.to_string();

              let dep_path = Self::resolve_dep_path(&dep, module_dir, context)?;

              dependencies.push(dep.clone());
              queue.push_back(dep_path);
            }
          }
        }
      } else {
        return Err("不支持 Script 模式".into());
      }

      let module_id = module_path.to_string_lossy().to_string();
      let module = Module::new(module_id.clone(), Some(dependencies));

      self.module_graph.add_single_module(module_id, module);
    }

    Ok(())
  }

  // make 完成后的检查阶段，但是这里先不实现了
  // pub fn finish(&self) {
  //   println!("\n[rust finish 阶段] 暂时跳过",);
  // }

  // module graph -> chunk graph
  pub async fn seal(&mut self) -> Result<(), String> {
    println!("\n[rust seal 阶段] module graph -> chunk graph");

    // 收集所有模块 ID
    let mut module_ids = Vec::new();
    for partial in &self.module_graph.partials {
      for module_id in partial.modules.keys() {
        module_ids.push(module_id.clone());
      }
    }

    println!("找到 {} 个模块：{:?}", module_ids.len(), module_ids);

    /**
     * 真实情况下会有一些分组策略，但是这里做简化，
     * 将所有模块放到一个 chunk 中
     */
    let chunk = Chunk {
      id: "main".to_string(),
      module_ids,
    };

    self.chunk_graph.chunks.push(chunk);
    Ok(())
  }
}
