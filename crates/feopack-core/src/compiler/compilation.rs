use crate::module_graph::{Module, ModuleGraph};
use crate::swc_compiler::SwcCompiler;
use std::path::Path;
use swc_ecma_ast::Program;
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

  // 产生 module graph (module + deps)
  pub async fn make(&mut self) -> Result<(), String> {
    /*
     * 这里 rspack 实际情况是把数据结构转为 EntryDependency, 然后通过 ModuleFactory 创建 Module
     * 另外还会开一个 Task Loop 做并行调度
     */
    let entry = self.options.entry.clone();
    let context = Path::new(&self.options.context);
    // PathBuf 类型可以跨平台
    let entry_path = context.join(&entry);

    println!("\n[rust make 阶段] 读取文件: {:?}", entry_path);

    // 使用 tokio::fs::read_to_string，因为是 async 函数
    let source = read_to_string(&entry_path)
      .await
      .map_err(|e| format!("读取文件失败 {:?}: {}", entry_path, e))?;

    println!("文件内容长度: {} 字节", source.len());

    let compiler = SwcCompiler::new();
    let ast = compiler.parse_js(entry_path.clone(), source)?;

    // 收集依赖并创建 Module
    let module_id = entry_path.to_string_lossy().to_string();
    let mut dependencies = Vec::new();

    match &ast {
      Program::Module(module) => {
        println!("\n解析到 {} 个语句/声明", module.body.len());

        // 收集所有 import 依赖
        for item in &module.body {
          use swc_ecma_ast::{ModuleDecl, ModuleItem};
          if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item {
            // Wtf8Atom 通过 as_str() 获取 &str
            if let Some(dep) = import.src.value.as_str() {
              let dep = dep.to_string();
              println!("  发现 import: {}", dep);
              dependencies.push(dep);
            }
          }
        }
      }
      Program::Script(_) => {
        return Err("不支持 Script 模式，只支持 Module 模式".to_string());
      }
    }

    // 创建 Module 并添加到 module graph
    let module = Module::new(module_id.clone(), Some(dependencies));
    self.module_graph.add_single_module(module_id, module);

    Ok(())
  }

  // make 完成后的检查阶段，但是这里先不实现了
  // pub fn finish(&self) {
  //   println!("\n[rust finish 阶段] 暂时跳过",);
  // }

  // module graph -> chunk graph
  pub fn seal(&mut self) {
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
  }
}
