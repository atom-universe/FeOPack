use super::{Compilation, ResolvedPath};
use crate::loader::inline_request;
use crate::loader::{split_loader_chain, JsLoaderRequest};
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
    println!("========\ncontext: {:?}\n", context);
    let entry_module_path = context.join(&self.options.entry);
    // id 其实就是 stringfied module path
    // 为什么要这么做呢？因为这里有 virtual import 等情况，会带 query 参数，形如 /abs/xxx?yyy=123 这样
    // 而这些事非标准的 Path，无法被 OS File Sys 解析，所以不能用 Pathbuf
    let entry_module_id = Self::create_module_id(&entry_module_path)?;
    println!("========\nentry_path: {:?}\n", entry_module_id);

    let mut module_id_queue: VecDeque<String> = VecDeque::new();
    // 朴素剪枝：当前只处理静态 import，所以 queue + visited 已经能表达最小 module graph 构建。
    let mut visited: HashSet<String> = HashSet::new();

    
    module_id_queue.push_back(entry_module_id);

    while let Some(module_id) = module_id_queue.pop_front() {
      if visited.contains(&module_id) {
        continue;
      }
      visited.insert(module_id.clone());

      // 构建一个 module 内部依赖的其他 modules
      let dep_module_ids = self.build_module(module_id).await?;
      for dep_module_path in dep_module_ids {
        module_id_queue.push_back(dep_module_path);
      }
    }

    Ok(())
  }

  async fn build_module(&mut self, module_id: String) -> Result<Vec<String>, String> {
    let context_path = self.options.context.clone();
    let context = Path::new(&context_path);
    let inline = inline_request::parse_inline_request(&module_id);
    let (module_resource_path, query) = inline
      .resource
      .split_once('?')
      .map(|(p, q)| (p, format!("?{}", q)))
      .unwrap_or_else(|| (inline.resource.as_str(), String::new()));

    let module_path = PathBuf::from(module_resource_path);
    let source = self
      .load_module_source(&module_path, &query, &inline)
      .await?;

    // 这里每次 build module 都临时创建 SwcCompiler。
    // 当初是为了快速绕开 source_map/引用生命周期问题；坏处是暂时共享不了 sourcemap。
    let compiler = SwcCompiler::new();
    let ast = compiler.parse_js(module_path.clone(), source.clone())?;

    // 字面意思，也就是依赖的模块
    // external 和 internal 的区别就是，后者进入 module graph，会参与打包，而前者不会——知道这个原理，实现 external 就方便了
    let mut dep_modules = Vec::new();
    let mut dep_module_paths = Vec::new();

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
          dep_modules.push(dep.clone());

          // external 依赖只记录在 dependencies 中，不进入本地构建队列。
          if let ResolvedPath::File(resolved_module) = Self::resolve_path(&dep, module_dir, context)? {
            dep_module_paths.push(resolved_module.module_id);
          }
        }
      }
    }

    // loader 已经在 make 阶段执行过，这里保存 transformed source。
    // code_generation 后续只消费这个 build result，不再重新读文件或重新跑 loader。
    self.module_sources.insert(module_id.clone(), source);

    let module = Module::new(module_id.clone(), Some(dep_modules));
    self.module_graph.add_single_module(module_id, module);

    Ok(dep_module_paths)
  }

  async fn load_module_source(
    &mut self,
    module_path: &PathBuf,
    query: &str,
    inline: &inline_request::InlineRequest,
  ) -> Result<String, String> {
    
    /// 注册后走 normalize，把无论是 js 的还是 rust 的 loaderchain 都转成一个 chain
    let loader_chain = self
      .loader_registry
      .resolve_chain(module_path, query, inline);
    let pitch_context = crate::loader::LoaderContext {
      resource_path: module_path.clone(),
      resource_query: query.to_string(),
      source: String::new(),
    };

    let source = if let Some(pitched_source) = self
      .loader_registry
      .run_pitch(&pitch_context, &loader_chain)?
    {
      pitched_source
    } else {
      self.read_resource_file(module_path).await?
    };

    // 统一下参数格式
    let normal_context = crate::loader::LoaderContext {
      resource_path: module_path.clone(),
      resource_query: query.to_string(),
      source: String::new(),
    };

    // 执行的时候拆出来，js 的走 ipc 丢回去给 node 侧跑 js loader runner 执行，rust 的就留在这边跑
    let (rust_loaders, js_loaders) = split_loader_chain(&loader_chain);
    let mut cur_source = source;

    if !rust_loaders.is_empty() {
      cur_source = self.loader_registry.run_normal(
        normal_context.clone(),
        &rust_loaders,
        cur_source,
      )?;
    }

    if !js_loaders.is_empty() {
      let runner = self
        .js_loader_runner
        .as_ref()
        .ok_or_else(|| "JS loader runner 初始化失败".to_string())?;

      let resource = format!(
        "{}{}{}",
        module_path.display(),
        query,
        ""
      );

      cur_source = runner(JsLoaderRequest {
        loaders: js_loaders,
        resource,
        source: cur_source,
        context: self.options.context.clone(),
      })?;
    }

    Ok(cur_source)
  }

  /// 读磁盘原文，同一次 compilation 内按 resource_path 去重。
  async fn read_resource_file(&mut self, module_path: &PathBuf) -> Result<String, String> {
    let cache_key = Self::normalize_path(module_path)?;

    if let Some(cached) = self.file_source_cache.get(&cache_key) {
      return Ok(cached.clone());
    }

    let content = read_to_string(&cache_key)
      .await
      .map_err(|e| format!("读取模块文件失败 {:?}: {}", cache_key, e))?;

    self.file_source_cache.insert(cache_key, content.clone());
    Ok(content)
  }
}
