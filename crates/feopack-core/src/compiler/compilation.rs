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

#[derive(Debug, Clone)]
pub struct GeneratedAsset {
  pub filename: String,
  pub source: String,
}

#[derive(Debug)]
pub struct Compilation {
  pub options: CompilationOptions,
  pub module_graph: ModuleGraph,
  pub chunk_graph: ChunkGraph,
  pub assets: Vec<GeneratedAsset>,
}

impl Compilation {
  pub fn new(options: CompilationOptions) -> Self {
    println!("\n\nCompilation new: {:?}\n\n", options);
    Self {
      options: options.clone(),
      module_graph: ModuleGraph::new(),
      chunk_graph: ChunkGraph::default(),
      assets: Vec::new(),
    }
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

              let dep_path = Self::resolve_path(&dep, module_dir, context)?;

              dependencies.push(dep.clone());
              queue.push_back(dep_path);
            }
          }
        }
      } else {
        return Err("不支持 Script 模式".into());
      }

      // 规范化路径，去除 ./ 等相对路径组件，使其可以直接用于文件读取
      let normalized_path = Self::normalize_path(&module_path)?;
      let module_id = normalized_path.to_string_lossy().to_string();
      let module = Module::new(module_id.clone(), Some(dependencies));

      self.module_graph.add_single_module(module_id, module);
    }

    Ok(())
  }

  // make 完成后的检查阶段，但是这里先不实现了
  // pub fn make_done(&self) {
  //   println!("\n[rust finish 阶段] 暂时跳过",);
  // }

  // module graph -> chunk graph
  pub async fn seal(&mut self) -> Result<(), String> {
    println!("\n[rust seal 阶段] module graph -> chunk graph");
    self.create_chunk_graph().await;
    // 给每一个 module 生成代码
    self.code_generation().await?;
    // 产生 bundle 并添加到 asset（但是写盘要放到 Compiler.emit_assets 中）
    // self.create_module_assets().await?;
    // 暂时不涉及处理非 js 文件
    // 如果要处理的话，这里后续需要走插件流程
    Ok(())
  }

  // pub async fn seal_done() {

  // }

  /**
   * 真实情况下会有一些分组策略，但是这里做简化，
   * 将所有模块放到一个 chunk 中
   */
  async fn create_chunk_graph(&mut self) {
    let mut module_ids = Vec::new();
    for partial in &self.module_graph.partials {
      for module_id in partial.modules.keys() {
        module_ids.push(module_id.clone());
      }
    }
    let chunk = Chunk {
      id: "main".to_string(),
      module_ids,
    };
    self.chunk_graph.chunks.push(chunk);

    eprintln!("modules: {:?}", self.module_graph.partials);
  }

  // 代码生成：为每个模块生成代码并创建 bundle
  async fn code_generation(&mut self) -> Result<(), String> {
    let context = Path::new(&self.options.context);
    let entry_path = context.join(&self.options.entry);
    let normalized_entry = Self::normalize_path(&entry_path)?;
    let entry_module_id = normalized_entry.to_string_lossy().to_string();

    for chunk in &self.chunk_graph.chunks {
      let mut modules_code = String::new();

      for module_id in &chunk.module_ids {
        // 读取模块源代码
        let source = read_to_string(module_id)
          .await
          .map_err(|e| format!("读取模块文件失败 {:?}: {}", module_id, e))?;

        //  "module_id": function(module, exports, require) { source }
        modules_code.push_str(&format!(
          r#""{}": function(module, exports, require) {{
  {}
  }},"#,
          module_id,
          // 缩进
          source
            .lines()
            .map(|line| format!("  {}", line))
            .collect::<Vec<_>>()
            .join("\n")
        ));
        modules_code.push('\n');
      }
      let bundle = format!(
        r#"(function(modules) {{
    const cache = {{}};
    function require(id) {{
      if (cache[id]) return cache[id].exports;
      const module = {{ exports: {{}} }};
      cache[id] = module;
      modules[id](module, module.exports, require);
      return module.exports;
    }}
    require("{}");
  }})({{
  {}
  }});"#,
        entry_module_id, modules_code
      );

      self.assets.push(GeneratedAsset {
        filename: self.options.output.filename.clone(),
        source: bundle,
      });
    }

    Ok(())
  }

  // 创建模块 assets（当前实现中已经在 code_generation 中完成）
  // async fn create_module_assets(&self) -> Result<(), String> {
  //   Ok(())
  // }

  // path.join
  fn normalize_path(path: &PathBuf) -> Result<PathBuf, String> {
    // 使用 components 过滤掉 CurDir (.)，保留其他组件
    // 将 /path/to/./file.js 转换为 /path/to/file.js
    let normalized: PathBuf = path
      .components()
      .filter(|c| !matches!(c, std::path::Component::CurDir))
      .collect();
    Ok(normalized)
  }

  fn resolve_path(dep: &str, module_dir: &Path, context: &Path) -> Result<PathBuf, String> {
    // 如果依赖路径以 . 或 .. 开头，相对于当前模块的目录解析
    let dep_path = if dep.starts_with('.') {
      module_dir.join(dep)
    } else {
      // 对于非相对路径（如 npm 包），使用 context
      context.join(dep)
    };

    // 规范化路径（去除 ./ 等组件）
    let normalized = Self::normalize_path(&dep_path)?;

    let final_path = if !normalized.exists() && normalized.extension().is_none() {
      let dep_path_with_js = normalized.with_extension("js");
      if dep_path_with_js.exists() {
        dep_path_with_js
      } else {
        normalized
      }
    } else {
      normalized
    };

    Ok(final_path)
  }
}
