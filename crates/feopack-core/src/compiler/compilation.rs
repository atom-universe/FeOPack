use crate::module_graph::{Module, ModuleGraph};
use crate::swc_compiler::{RawImportRecord, ResolvedImportRecord, SwcCompiler};
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

#[derive(Debug, Clone)]
struct CodegenModule {
  id: String,
  source: String,
  imports: Vec<ResolvedImportRecord>,
}

#[derive(Debug, Clone)]
enum ResolvedPath {
  File(PathBuf),
  External(String),
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
   * TODO: 等我完全处理好了基本的打包流程后，再回头看下 rspack 是怎么做的
   */
  pub async fn make(&mut self) -> Result<(), String> {
    let context = Path::new(&self.options.context);
    let entry_path = context.join(&self.options.entry);

    // 朴素剪枝
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

      // 为什么每次都重复用一个？（当初肯定是为了图省事，搞不定 rust 的引用最后妥协了）
      // TODO: 这里底层是这样的：let source_file = self.source_map.new_source_file(filename, source);
      // 所以如果是现在这样的写法，恐怕无法共享sourcemap（不过这有什么坏处呢？）

      let compiler = SwcCompiler::new();
      let ast = compiler.parse_js(module_path.clone(), source)?;

      let mut dependencies = Vec::new();

      if let Program::Module(module) = ast {
        let module_dir = module_path
          .parent()
          .ok_or_else(|| format!("无法获取模块目录: {:?}", module_path))?;

        for item in module.body {
          if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item {
            if let Some(dep) = import.src.value.as_str() {
              let dep = dep.to_string();

              dependencies.push(dep.clone());
              if let ResolvedPath::File(dep_path) = Self::resolve_path(&dep, module_dir, context)? {
                queue.push_back(dep_path);
              }
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
    println!(
      "\n[rust seal 阶段] module graph -> chunk graph:\n {:?}\n",
      self.module_graph.partials
    );
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
      // pair list，(module id, source)
      let mut module_sources = Vec::new();

      for module_id in &chunk.module_ids {
        // 读取模块源代码
        let source = read_to_string(module_id)
          .await
          .map_err(|e| format!("读取模块文件失败 {:?}: {}", module_id, e))?;
        println!("模块源代码: {:?}", source);
        module_sources.push((module_id.clone(), source));
      }

      // 这里其实是因为我也不太熟悉 rust 的一些特性
      // 出现各种跨线程的抽象问题
      // 所以直接选择了简单省事地在每个 chunk 处理过程中都开一个 compiler 来处理 ast
      let compiler = SwcCompiler::new();
      let mut codegen_modules: Vec<CodegenModule> = Vec::new();
      // 外部依赖不进入本地 module graph，运行时交给 Node require 处理。
      let mut external_module_ids = HashSet::new();

      for (module_id, source) in module_sources {
        // 这里的 PathBuf 是。。。
        let ast: Program = compiler.parse_js(PathBuf::from(&module_id), source)?;
        let raw_imports = compiler.collect_imports(&ast)?;
        // println!("raw_imports: {:#?}", raw_imports);
        let imports = self.resolve_imports(&module_id, raw_imports)?;
        // println!("resolved_imports: {:#?}", imports);
        for import in &imports {
          if import.external {
            external_module_ids.insert(import.module_id.clone());
          }
        }
        let transformed_ast = compiler.transform_module_ast(ast, &imports)?;

        let generated_source = compiler.emit_module(&transformed_ast)?;
        // println!("ast 生成源码: {}", generated_source);
        // println!("imports: {:#?}", imports);

        codegen_modules.push(CodegenModule {
          id: module_id,
          source: generated_source,
          imports,
        });
      }

      for module_id in external_module_ids {
        codegen_modules.push(CodegenModule {
          id: module_id.clone(),
          source: format!(
            r#"const __feopack_external__ = require("{}");
Object.assign(__feopack_exports__, __feopack_external__);
__feopack_import__.d(__feopack_exports__, {{
    default: () => __feopack_external__
}});"#,
            module_id
          ),
          imports: Vec::new(),
        });
      }

      let source = Self::render_chunk(&entry_module_id, &codegen_modules);

      // for codegen_module in &codegen_modules {
      //   println!("  module id: {}", codegen_module.id);
      //   println!("  source length: {}", codegen_module.source.len());

      //   for import in &codegen_module.imports {
      //     println!("########################");
      //     println!(
      //       "  import:\n    local: {}\n    imported: {}\n    request: {}\n    module_id: {}",
      //       import.local, import.imported, import.request, import.module_id
      //     );
      //     println!("########################");
      //   }
      // }

      self.assets.push(GeneratedAsset {
        filename: self.options.output.filename.clone(),
        source,
      });
    }
    Ok(())
  }

  // 之前是 cjs 风格，现在换到 esm 风格
  // 谢谢 ai 帮忙处理（写了几次都有问题）
  fn render_chunk(entry_module_id: &str, modules: &[CodegenModule]) -> String {
    let mut modules_code = String::new();

    for module in modules {
      let module_source = module
        .source
        .lines()
        .map(|line| format!("    {}", line))
        .collect::<Vec<_>>()
        .join("\n");

      modules_code.push_str(&format!(
        r#"  "{}": (__feopack_module__, __feopack_exports__, __feopack_import__) => {{
{}
  }},
"#,
        module.id, module_source
      ));
    }
    // !TODO: 可以包一层 IIFE，防止这里 feopack_modules 和 feopack_cache 污染
    // {}
    format!(
      r#"
const __feopack_modules__ = {{{}}};
const __feopack_cache__ = {{}};

function __feopack_import__(id) {{
  if (__feopack_cache__[id]) {{
    return __feopack_cache__[id].exports;
  }}

  const __feopack_module__ = {{ exports: {{}} }};
  __feopack_cache__[id] = __feopack_module__;
  __feopack_modules__[id](__feopack_module__, __feopack_module__.exports, __feopack_import__);

  return __feopack_module__.exports;
}}

__feopack_import__.d = (exports, definition) => {{
  for (const key in definition) {{
    if (
      Object.prototype.hasOwnProperty.call(definition, key) &&
      !Object.prototype.hasOwnProperty.call(exports, key)
    ) {{
      Object.defineProperty(exports, key, {{
        enumerable: true,
        get: definition[key],
      }});  
    }}
  }}
}};

__feopack_import__("{}");
"#,
      modules_code, entry_module_id
    )
  }

  fn resolve_imports(
    &self,
    module_id: &str,
    raw_imports: Vec<RawImportRecord>,
  ) -> Result<Vec<ResolvedImportRecord>, String> {
    let context = Path::new(&self.options.context);
    let module_path = PathBuf::from(module_id);
    let module_dir = module_path
      .parent()
      .ok_or_else(|| format!("无法获取模块目录: {}", module_id))?;

    // 一个很有意思的事情：为什么不用 map 而是用 vec 呢？
    // 因为这里 import 是有顺序的，所以用 vec 更合适，妙啊
    let mut imports = Vec::new();

    for raw_import in raw_imports {
      let resolved_path = Self::resolve_path(&raw_import.request, module_dir, context)?;
      let external = matches!(resolved_path, ResolvedPath::External(_));
      let dep_module_id = match resolved_path {
        ResolvedPath::File(dep_path) => Self::normalize_path(&dep_path)?
          .to_string_lossy()
          .to_string(),
        ResolvedPath::External(module_id) => module_id,
      };

      imports.push(ResolvedImportRecord {
        local: raw_import.local,
        imported: raw_import.imported,
        request: raw_import.request,
        module_id: dep_module_id,
        external,
      });
    }

    Ok(imports)
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

  fn resolve_path(dep: &str, module_dir: &Path, _context: &Path) -> Result<ResolvedPath, String> {
    if Self::is_external_request(dep) {
      return Ok(ResolvedPath::External(dep.to_string()));
    }

    // 如果依赖路径以 . 或 .. 开头，相对于当前模块的目录解析
    let dep_path = module_dir.join(dep);

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

    Ok(ResolvedPath::File(final_path))
  }

  pub fn is_external_request(request: &str) -> bool {
    !request.starts_with('.')
  }
}
