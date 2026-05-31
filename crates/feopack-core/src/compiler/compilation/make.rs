use super::{Compilation, ResolvedPath};
use crate::module_graph::Module;
use crate::swc_compiler::SwcCompiler;
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use swc_ecma_ast::{ModuleDecl, ModuleItem, Program};
use tokio::fs::read_to_string;

impl Compilation {
  /*
   * 这里 rspack 实际情况是把数据结构转为 EntryDependency, 然后通过 ModuleFactory 创建 Module
   * 另外还会开一个 Task Loop 做并行调度, 这里就用 BFS 大致模拟
   * TODO: 等我完全处理好了基本的打包流程后，再回头看下 rspack 是怎么做的
   */
  pub(crate) async fn build_module_graph(&mut self) -> Result<(), String> {
    let context = Path::new(&self.options.context);
    let entry_path = context.join(&self.options.entry);

    // 朴素剪枝：当前只处理静态 import，所以 queue + visited 已经能表达最小 module graph 构建。
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();

    queue.push_back(entry_path);

    while let Some(module_path) = queue.pop_front() {
      if visited.contains(&module_path) {
        continue;
      }
      visited.insert(module_path.clone());

      let dep_paths = self.build_module(module_path).await?;
      for dep_path in dep_paths {
        queue.push_back(dep_path);
      }
    }

    Ok(())
  }

  async fn build_module(&mut self, module_path: PathBuf) -> Result<Vec<PathBuf>, String> {
    let context = Path::new(&self.options.context);
    let source = self.load_module_source(&module_path).await?;

    // 这里每次 build module 都临时创建 SwcCompiler。
    // 当初是为了快速绕开 source_map/引用生命周期问题；坏处是暂时共享不了 sourcemap。
    let compiler = SwcCompiler::new();
    let ast = compiler.parse_js(module_path.clone(), source.clone())?;

    let mut dependencies = Vec::new();
    let mut dep_paths = Vec::new();

    let Program::Module(module) = ast else {
      return Err("不支持 Script 模式".into());
    };

    let module_dir = module_path
      .parent()
      .ok_or_else(|| format!("无法获取模块目录: {:?}", module_path))?;

    for item in module.body {
      if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item {
        if let Some(dep) = import.src.value.as_str() {
          let dep = dep.to_string();
          dependencies.push(dep.clone());

          // external 依赖只记录在 dependencies 中，不进入本地构建队列。
          if let ResolvedPath::File(dep_path) = Self::resolve_path(&dep, module_dir, context)? {
            dep_paths.push(dep_path);
          }
        }
      }
    }

    let module_id = Self::create_module_id(&module_path)?;
    // loader 已经在 make 阶段执行过，这里保存 transformed source。
    // code_generation 后续只消费这个 build result，不再重新读文件或重新跑 loader。
    self.module_sources.insert(module_id.clone(), source);

    let module = Module::new(module_id.clone(), Some(dependencies));
    self.module_graph.add_single_module(module_id, module);

    Ok(dep_paths)
  }

  async fn load_module_source(&self, module_path: &PathBuf) -> Result<String, String> {
    let source = read_to_string(module_path)
      .await
      .map_err(|e| format!("读取模块文件失败 {:?}: {}", module_path, e))?;

    self.loader_registry.run(module_path.clone(), source)
  }
}
