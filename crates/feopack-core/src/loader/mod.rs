// 内置的 rust loader
use std::collections::HashMap;
use std::path::PathBuf;

pub mod text_loader;
pub mod meow_loader_v1;

// test: '/\.test$/',
// use_loaders: [
//   'ts-loader',
//   'babel-loader',
// ]
#[derive(Debug)]
pub struct LoaderRule {
  pub test: String,
  pub used_loaders: Vec<String>,
}

#[derive(Debug)]
pub struct LoaderContext {
  pub resource_path: PathBuf,
  pub source: String,
  // import xx from 'xx.vue'
  // virtual request 化后：
  // import xx from 'xx.vue?type=template'
  // import xx from 'xx.vue?type=script'
  // import xx from 'xx.vue?type=style'
  // pub resource_query: String,
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

  // 1. 遍历资源路径，然后用后缀去匹配 rule.test
  // 2. 如果匹配到，则使用 rule.use_loader 中的 loader 进行处理
  // 3. 如果没匹配到，就直接返回原始文件的内容，不做处理
  pub fn run(&self, resource_path: PathBuf, source: String) -> Result<String, String> {
    let Some(rule) = self.rules.iter().find(|rule| {
      resource_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| rule.test.trim_start_matches('.') == ext)
        .unwrap_or(false)
    }) else {
      return Ok(source);
    };

    let mut cur_source = source;

    // rev 颠倒一下顺序，这样让用户感知到的 loader 是从右往左执行，和 webpack 一致
    for loader_name in rule.used_loaders.iter().rev() {
      let loader = self
        .loaders
        .get(loader_name)
        .ok_or_else(|| format!("找不到 loader: {}", loader_name))?;

      // loader 的其实就是输入文件内容+路径，输出新的文件内容，仅此而已。。。
      cur_source = loader(LoaderContext {
        resource_path: resource_path.clone(),
        source: cur_source,
      })?;
    }

    Ok(cur_source)
  }
}
