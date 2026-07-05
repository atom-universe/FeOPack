// 内置的 rust loader
use std::collections::HashMap;
use std::path::PathBuf;

use inline_request::InlineRequest;
use meow_loader_v3::resolve_meow_v3_chain;

pub mod text_loader;
pub mod meow_loader_v1;
pub mod meow_loader_v2;
pub mod meow_loader_v3;
pub mod typescript_loader;
pub mod inline_request;
pub mod js_bridge;

pub use js_bridge::{is_js_loader, split_loader_chain, JsLoaderRequest, JsLoaderRunner};

// test: '/\.test$/',
// use_loaders: [
//   'ts-loader',
//   'babel-loader',
// ]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoaderEnforce {
  #[default]
  Normal,
  Pre,
  Post,
}

#[derive(Debug)]
pub struct LoaderRule {
  pub test: String,
  pub resource_query: String,
  pub used_loaders: Vec<String>,
  pub enforce: LoaderEnforce,
}

#[derive(Debug, Clone)]
pub struct LoaderContext {
  // 这里是一个很讲究的小巧思，resource 偏向于指的文件路径等
  // 而 source 则侧重指文件的内容
  pub resource_path: PathBuf,
  pub source: String,
  // import xx from 'xx.vue'
  // virtual request 化后：
  // import xx from 'xx.vue?type=template'
  // import xx from 'xx.vue?type=script'
  // import xx from 'xx.vue?type=style'
  pub resource_query: String,
}

pub type NormalFn = fn(LoaderContext) -> Result<String, String>;
pub type PitchFn = fn(&LoaderContext) -> Result<PitchResult, String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PitchResult {
  /// 等同 webpack pitch 返回 undefined：继续后续 pitch，并在需要时读盘
  Continue,
  /// 等同 pitch 返回 string：短路，跳过读盘与剩余 pitch
  ShortCircuit(String),
}

#[derive(Debug, Clone, Copy)]
pub struct Loader {
  pub pitch: Option<PitchFn>,
  pub normal: NormalFn,
}

impl Loader {
  pub fn normal_only(normal: NormalFn) -> Self {
    Self {
      pitch: None,
      normal,
    }
  }

  pub fn with_pitch(pitch: PitchFn, normal: NormalFn) -> Self {
    Self {
      pitch: Some(pitch),
      normal,
    }
  }
}

#[derive(Debug)]
pub struct LoaderRegistry {
  loaders: HashMap<String, Loader>,
  /// config module.rules（优先于内置 rules）
  user_rules: Vec<LoaderRule>,
  rules: Vec<LoaderRule>,
}

impl LoaderRegistry {
  pub fn new() -> Self {
    Self {
      loaders: HashMap::new(),
      user_rules: Vec::new(),
      rules: Vec::new(),
    }
  }

  pub fn set_user_rules(&mut self, rules: Vec<LoaderRule>) {
    self.user_rules = rules;
  }

  pub fn register_loader(&mut self, name: String, loader: Loader) {
    self.loaders.insert(name.to_string(), loader);
  }

  pub fn add_rule(&mut self, rule: LoaderRule) {
    self.rules.push(rule);
  }

  pub fn resolve_chain(
    &self,
    resource_path: &PathBuf,
    resource_query: &str,
    inline: &InlineRequest,
  ) -> Vec<String> {
        // loader chain 到底解决什么问题呢？（可以参考 crates/feopack-core/src/compiler/compilation/mod.rs 的注释）
    // meow-loader-v2 这种 loader 会产生 virtual requests, 可能需要多类 loader 来处理
    // 本来是需要手动为每种情况制定 loader 的配方（我的老天，我认为这真是一个绝妙的描述💗 
    // 而 loader chain 要做的就是，根据 virtual request 的情况，自动编排 loader 的配方
    // 与此同时，为了更好地编排，还约定了一套新的语法——我们叫做 inline loader, 也就是在 import 加入特殊的标记
    // (注意，这里标准 rspack/webpack 的 loader 有 pitch 和 normal，但是我们这里暂时还没有这些东西)
    // （如果有 pitch 的话，我们可以不用把 loader chain 暴露到 virtual request 里；不过，就当前效果而言，大差不差，用户是不感知的，等后续我们遇到问题的时候再继续吧）
    // 例如（AI总结的，大概看懂含义即可，不必细究）：
    //
    //   loader1!loader2!resource：
    //   loader 名用 ! 分隔，写在 resource 前面；执行顺序从右往左
    //
    //   -!loader1!loader2!resource
    //   前缀语义（对齐 webpack）：
    //     !   → 跳过 config normal
    //     -!  → 跳过 config pre + normal（post 仍可能参与）
    //     !!  → 跳过 config pre + normal + post
    //
    //   ./index.ts
    //   typescript-loader!./index.ts
    //   -!meow-wrap-script-export!typescript-loader!meow-extract-script!./index.meow-v2?type=script&lang=ts
    //
    // 具体而言，比如 meow-loader-v2-main，生成的 import 示例：
    //   import __script__ from '-!meow-wrap-script-export!typescript-loader!meow-extract-script!./index.meow-v2?type=script&lang=ts';
    // 反面例子（没有 inline loader 时）：
    //   import 里只有 virtual request，loader 配方得全部写在 mod.rs 的 rule 里：
    //
    //   // meow-main 生成的 import（旧写法）
    //   import __script__ from './index.meow-v2?type=script&lang=ts';
    //   import __template__ from './index.meow-v2?type=template';
    //   import __style__ from './index.meow-v2?type=style&scoped';
    //
    //   // 同时，还要为每一种 query 各配一条 rule（lang=ts / lang=js / scoped ... 各一条）
    //   1: {
    //     test: ".meow-v2",
    //     resource_query: "?type=script&lang=ts",
    //     used_loaders: vec!["meow-wrap-script-export", "typescript-loader", "meow-extract-script"],
    //   }
    //   2: { test: ".meow-v2", resource_query: "?type=template", used_loaders: vec!["meow-wrap-template-export", "meow-extract-template"] },
    //   3: { test: ".meow-v2", resource_query: "?type=style&scoped", used_loaders: vec!["meow-wrap-style-export", "meow-scope-style", "meow-extract-style"] },
    //   // ... 每多一种 block 属性组合，就多一条
    //
    let mut loader_chain = Vec::new();

    if !inline.loaders.is_empty() {
      loader_chain.extend(inline.loaders.iter().cloned());
    }

    // meow-v3（vue-loader 风格）：用户只配一条 rule；子 block 链由 resource_query 在 loader 内拼出
    if resource_path
      .extension()
      .and_then(|ext| ext.to_str())
      .is_some_and(|ext| ext == "meow-v3")
    {
      if let Ok(v3_chain) = resolve_meow_v3_chain(resource_query) {
        loader_chain.extend(v3_chain);
      }
      return loader_chain;
    }

    // 用户 config module.rules（优先于内置 enforce rules）
    if let Some(rule) = self
      .user_rules
      .iter()
      .find(|rule| Self::rule_matches(rule, resource_path, resource_query))
    {
      loader_chain.extend(rule.used_loaders.iter().cloned());
      return loader_chain;
    }

    for enforce in [LoaderEnforce::Pre, LoaderEnforce::Normal, LoaderEnforce::Post] {
      if inline.config_override.allows_config_rule(enforce) {
        if let Some(rule) = self.rules.iter().find(|rule| {
          rule.enforce == enforce
            && resource_path
              .extension()
              .and_then(|ext| ext.to_str())
              .map(|ext| {
                rule.test.trim_start_matches('.') == ext && rule.resource_query == resource_query
              })
              .unwrap_or(false)
        }) {
          loader_chain.extend(rule.used_loaders.iter().cloned());
        }
      }
    }

    loader_chain
  }

  fn rule_matches(rule: &LoaderRule, resource_path: &PathBuf, resource_query: &str) -> bool {
    resource_path
      .extension()
      .and_then(|ext| ext.to_str())
      .is_some_and(|ext| rule.test.trim_start_matches('.') == ext)
      && rule.resource_query == resource_query
  }

  /// pitch 阶段：从左到右；若某个 pitch 返回 ShortCircuit，则跳过后续 pitch 与读盘
  pub fn run_pitch(
    &self,
    context: &LoaderContext,
    loader_chain: &[String],
  ) -> Result<Option<String>, String> {
    for loader_name in loader_chain {
      let Some(pitch) = self
        .loaders
        .get(loader_name)
        .and_then(|loader| loader.pitch)
      else {
        continue;
      };

      match pitch(context)? {
        PitchResult::Continue => {}
        PitchResult::ShortCircuit(source) => return Ok(Some(source)),
      }
    }

    Ok(None)
  }

  /// normal 阶段：从右到左 transform source
  pub fn run_normal(
    &self,
    context: LoaderContext,
    loader_chain: &[String],
    source: String,
  ) -> Result<String, String> {
    if loader_chain.is_empty() {
      return Ok(source);
    }

    let mut cur_source = source;

    for loader_name in loader_chain.iter().rev() {
      let loader = self
        .loaders
        .get(loader_name)
        .ok_or_else(|| format!("找不到 loader: {}", loader_name))?;

      cur_source = (loader.normal)(LoaderContext {
        resource_path: context.resource_path.clone(),
        resource_query: context.resource_query.clone(),
        source: cur_source,
      })?;
    }

    Ok(cur_source)
  }

  /// pitch →（必要时读盘）→ normal
  pub fn run(
    &self,
    resource_path: PathBuf,
    resource_query: String,
    source: String,
    inline: &InlineRequest,
  ) -> Result<String, String> {
    let loader_chain = self.resolve_chain(&resource_path, &resource_query, inline);

    let pitch_context = LoaderContext {
      resource_path: resource_path.clone(),
      resource_query: resource_query.clone(),
      source: String::new(),
    };

    let initial_source = if let Some(pitched_source) = self.run_pitch(&pitch_context, &loader_chain)? {
      pitched_source
    } else {
      source
    };

    self.run_normal(
      LoaderContext {
        resource_path,
        resource_query,
        source: String::new(),
      },
      &loader_chain,
      initial_source,
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicUsize, Ordering};

  static PITCH_CALLS: AtomicUsize = AtomicUsize::new(0);

  fn counting_pitch(_ctx: &LoaderContext) -> Result<PitchResult, String> {
    PITCH_CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(PitchResult::Continue)
  }

  fn short_circuit_pitch(_ctx: &LoaderContext) -> Result<PitchResult, String> {
    Ok(PitchResult::ShortCircuit("from pitch".to_string()))
  }

  fn append_a(mut ctx: LoaderContext) -> Result<String, String> {
    ctx.source.push('A');
    Ok(ctx.source)
  }

  fn append_b(mut ctx: LoaderContext) -> Result<String, String> {
    ctx.source.push('B');
    Ok(ctx.source)
  }

  #[test]
  fn pitch_runs_left_to_right_before_normal() {
    PITCH_CALLS.store(0, Ordering::SeqCst);

    let mut registry = LoaderRegistry::new();
    registry.register_loader(
      "pitcher".to_string(),
      Loader::with_pitch(counting_pitch, append_a),
    );
    registry.register_loader("worker".to_string(), Loader::normal_only(append_b));
    registry.add_rule(LoaderRule {
      test: ".txt".to_string(),
      resource_query: String::new(),
      used_loaders: vec!["pitcher".to_string(), "worker".to_string()],
      enforce: LoaderEnforce::Normal,
    });

    let inline = InlineRequest::default();
    let output = registry
      .run(
        PathBuf::from("./demo.txt"),
        String::new(),
        "x".to_string(),
        &inline,
      )
      .expect("loader run");

    assert_eq!(PITCH_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(output, "xBA");
  }

  fn pass_through(ctx: LoaderContext) -> Result<String, String> {
    Ok(ctx.source)
  }

  #[test]
  fn pitch_short_circuit_skips_initial_source() {
    let mut registry = LoaderRegistry::new();
    registry.register_loader(
      "pitcher".to_string(),
      Loader::with_pitch(short_circuit_pitch, pass_through),
    );
    registry.register_loader("worker".to_string(), Loader::normal_only(append_b));
    registry.add_rule(LoaderRule {
      test: ".txt".to_string(),
      resource_query: String::new(),
      used_loaders: vec!["pitcher".to_string(), "worker".to_string()],
      enforce: LoaderEnforce::Normal,
    });

    let inline = InlineRequest::default();
    let output = registry
      .run(
        PathBuf::from("./demo.txt"),
        String::new(),
        "ignored".to_string(),
        &inline,
      )
      .expect("loader run");

    assert_eq!(output, "from pitchB");
  }
}
