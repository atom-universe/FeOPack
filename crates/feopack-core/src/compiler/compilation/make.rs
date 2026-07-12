use super::super::normal_module_factory::NormalModuleCreateData;
use super::{Compilation, ResolvedPath};
use crate::loader::{is_js_loader, JsLoaderRequest, LoaderContext};
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
    let create_data = self
      .normal_module_factory
      .create(module_id, &self.loader_registry);
    let module_path = create_data.resource_path.clone();
    let source = self.load_module_source(&create_data).await?;

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
          if let ResolvedPath::File(resolved_module) =
            Self::resolve_path(&dep, module_dir, context)?
          {
            dep_module_paths.push(resolved_module.module_id);
          }
        }
      }
    }

    // loader 已经在 make 阶段执行过，这里保存 transformed source。
    // code_generation 后续只消费这个 build result，不再重新读文件或重新跑 loader。
    self
      .module_sources
      .insert(create_data.module_id.clone(), source);

    let module = Module::new(create_data.module_id.clone(), Some(dep_modules));
    self
      .module_graph
      .add_single_module(create_data.module_id, module);

    Ok(dep_module_paths)
  }

  async fn load_module_source(
    &mut self,
    create_data: &NormalModuleCreateData,
  ) -> Result<String, String> {
    let module_path = &create_data.resource_path;
    let query = &create_data.resource_query;
    let loader_chain = &create_data.loaders;
    let pitch_context = LoaderContext {
      resource_path: module_path.clone(),
      resource_query: query.to_string(),
      source: String::new(),
    };

    let normal_context = LoaderContext {
      resource_path: module_path.clone(),
      resource_query: query.to_string(),
      source: String::new(),
    };

    // 因为 pitch 阶段可以 return 短路掉，所以得记录下结束位置
    // 不过一定要注意啊，触发 pitch 短路的那个 loader 本身，不会参与 normal 阶段的运行
    let (source, normal_start_index) = self
      .run_mixed_pitch_chain(&pitch_context, loader_chain)
      .await?;

    let source = if let Some(source) = source {
      source
    } else {
      self.read_resource_file(module_path).await?
    };

    self
      .run_mixed_normal_chain(normal_context, loader_chain, normal_start_index, source)
      .await
  }

  async fn run_mixed_pitch_chain(
    &self,
    context: &LoaderContext,
    loader_chain: &[String],
  ) -> Result<(Option<String>, Option<usize>), String> {
    let mut index = 0usize;

    while index < loader_chain.len() {
      // 从左向右，--------> 前进～
      let loader_name = &loader_chain[index];

      // 统计连续的 js loader 的 index 区间，打包一起发给 node
      //（感觉像是在做 力扣。。。
      if is_js_loader(loader_name) {
        let segment_start = index;
        while index < loader_chain.len() && is_js_loader(&loader_chain[index]) {
          index += 1;
        }

        let result = self
          .run_js_pitch_segment(context, &loader_chain[segment_start..index])
          .await?;

        if result.short_circuit {
          // 如果发生短路，把右边界设置为短路 loader 的那个 index
          // 这里的 local 是指的这段 segement list 的 index
          let local_index = result
            .pitched_loader_index
            .ok_or_else(|| "JS pitch 短路时缺少 pitched_loader_index".to_string())?;
          let short_circuit_index = segment_start + local_index;

          // 这里 checked_sub(1) 就是 max(0, short_circuit_index - 1)
          let normal_start = short_circuit_index.checked_sub(1);
          return Ok((Some(result.source), normal_start));
        }

        continue;
      }

      // TODO: 说实话这个代码写得很丑陋，后续有机会重构下
      // 最好是这个函数里面只有 run_pitch 和 run_normal 两个主要调用，其他的代码都放到外面或者别的文件里面
      let pitch_result = self
        .loader_registry
        .run_pitch(context, &[loader_name.clone()])?;
      if let Some(source) = pitch_result {
        let normal_start = index.checked_sub(1);
        return Ok((Some(source), normal_start));
      }

      index += 1;
    }

    Ok((None, loader_chain.len().checked_sub(1)))
  }

  async fn run_mixed_normal_chain(
    &self,
    context: LoaderContext,
    loader_chain: &[String],
    start_index: Option<usize>,
    source: String,
  ) -> Result<String, String> {
    let Some(start_index) = start_index else {
      return Ok(source);
    };

    if loader_chain.is_empty() {
      return Ok(source);
    }

    let mut cur_source = source;
    let mut pending_js_loaders = Vec::new();

    for loader_name in loader_chain[..=start_index].iter().rev() {
      if is_js_loader(loader_name) {
        pending_js_loaders.push(loader_name.clone());
        continue;
      }

      if !pending_js_loaders.is_empty() {
        cur_source = self
          .run_js_loader_segment(&context, &pending_js_loaders, cur_source)
          .await?;
        pending_js_loaders.clear();
      }

      cur_source =
        self
          .loader_registry
          .run_normal(context.clone(), &[loader_name.clone()], cur_source)?;
    }

    if !pending_js_loaders.is_empty() {
      cur_source = self
        .run_js_loader_segment(&context, &pending_js_loaders, cur_source)
        .await?;
    }

    Ok(cur_source)
  }

  async fn run_js_pitch_segment(
    &self,
    context: &LoaderContext,
    js_loaders: &[String],
  ) -> Result<crate::loader::JsLoaderRunResult, String> {
    let runner = self
      .js_loader_runner
      .as_ref()
      .ok_or_else(|| "JS loader runner 初始化失败".to_string())?;

    let resource = format!(
      "{}{}",
      context.resource_path.display(),
      context.resource_query
    );

    runner(JsLoaderRequest {
      loader_state: "pitching".to_string(),
      loaders: js_loaders.to_vec(),
      resource,
      source: String::new(),
      // 为什么不继续叫 context?
      // 因为这个命名太烂了，
      // 正常人都会以为这里说的是 loaderContext，谁能想到指的是 projectRoot 啊
      project_root: self.options.context.clone(),
    })
    .await
  }

  async fn run_js_loader_segment(
    &self,
    context: &LoaderContext,
    pending_js_loaders: &[String],
    source: String,
  ) -> Result<String, String> {
    let runner = self
      .js_loader_runner
      .as_ref()
      .ok_or_else(|| "JS loader runner 初始化失败".to_string())?;

    // 倒序扫描 normal 链时，收集到的连续 JS 段顺序是反的；交给 Node 前要翻回原始链顺序。
    let js_loaders = pending_js_loaders.iter().rev().cloned().collect();
    let resource = format!(
      "{}{}",
      context.resource_path.display(),
      context.resource_query
    );

    let result = runner(JsLoaderRequest {
      loader_state: "normal".to_string(),
      loaders: js_loaders,
      resource,
      source,
      project_root: self.options.context.clone(),
    })
    .await?;

    if result.short_circuit {
      return Err("normal 阶段的 JS loader 不应返回 short-circuit".to_string());
    }

    Ok(result.source)
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
