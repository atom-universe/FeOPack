use super::{
  JsLoaderRequest, JsLoaderRunResult, JsLoaderRunner, LoaderContext, LoaderRegistry, is_js_loader,
};

pub(crate) struct LoaderRunner<'a> {
  loader_registry: &'a LoaderRegistry,
  js_loader_runner: Option<&'a JsLoaderRunner>,
  project_root: &'a str,
}

impl<'a> LoaderRunner<'a> {
  pub(crate) fn new(
    loader_registry: &'a LoaderRegistry,
    js_loader_runner: Option<&'a JsLoaderRunner>,
    project_root: &'a str,
  ) -> Self {
    Self {
      loader_registry,
      js_loader_runner,
      project_root,
    }
  }

  pub(crate) async fn run_pitch_chain(
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

  pub(crate) async fn run_normal_chain(
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
  ) -> Result<JsLoaderRunResult, String> {
    let runner = self
      .js_loader_runner
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
      project_root: self.project_root.to_string(),
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
      project_root: self.project_root.to_string(),
    })
    .await?;

    if result.short_circuit {
      return Err("normal 阶段的 JS loader 不应返回 short-circuit".to_string());
    }

    Ok(result.source)
  }
}
