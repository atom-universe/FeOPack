// 内置的 rust loader
use std::collections::HashMap;
use std::path::PathBuf;

use inline_request::InlineRequest;

pub mod text_loader;
pub mod meow_loader_v1;
pub mod meow_loader_v2;
pub mod typescript_loader;
pub mod inline_request;

// test: '/\.test$/',
// use_loaders: [
//   'ts-loader',
//   'babel-loader',
// ]
#[derive(Debug)]
pub struct LoaderRule {
  pub test: String,
  pub resource_query: String,
  pub used_loaders: Vec<String>,
}

#[derive(Debug)]
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

pub type LoaderFn = fn(LoaderContext) -> Result<String, String>;

#[derive(Debug)]
pub struct LoaderRegistry {
  loaders: HashMap<String, LoaderFn>,
  rules: Vec<LoaderRule>,
}

impl LoaderRegistry {
  pub fn new() -> Self {
    Self {
      loaders: HashMap::new(),
      rules: Vec::new(),
    }
  }

  // loader 注册也是一个比较简单的事情，也就是给 loader 一个名字，然后把具体的处理函数注册进 map
  // 这个方法的意义就是注册内置的 loader （毕竟用户没地方输入这种配置，得内部完成）
  pub fn register_loader(&mut self, name: String, loader: LoaderFn) {
    self.loaders.insert(name.to_string(), loader);
  }

  pub fn add_rule(&mut self, rule: LoaderRule) {
    self.rules.push(rule);
  }

  // 1. 有 inline loader 时优先拼进执行链
  // 2. `-!` 表示只跑 inline loader，跳过 rule 匹配
  // 3. 否则再按后缀 + resource_query 匹配 rule
  pub fn run(
    &self,
    resource_path: PathBuf,
    resource_query: String,
    source: String,
    inline: &InlineRequest,
  ) -> Result<String, String> {
    let mut cur_source = source;
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
    //   -!前缀：只跑 inline 里的 loader，跳过 mod.rs 的 rule 匹配
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

    if !inline.inline_only {
      if let Some(rule) = self.rules.iter().find(|rule| {
        resource_path
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

    if loader_chain.is_empty() {
      return Ok(cur_source);
    }

    // rev 颠倒一下顺序，这样让用户感知到的 loader 是从右往左执行，和 webpack 一致
    for loader_name in loader_chain.iter().rev() {
      let loader = self
        .loaders
        .get(loader_name)
        .ok_or_else(|| format!("找不到 loader: {}", loader_name))?;

      cur_source = loader(LoaderContext {
        resource_path: resource_path.clone(),
        resource_query: resource_query.clone(),
        source: cur_source,
      })?;
    }

    Ok(cur_source)
  }
}
