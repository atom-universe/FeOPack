use std::sync::Arc;

/// Rust → Node 执行 JS loader 链时的请求体
#[derive(Debug, Clone)]
pub struct JsLoaderRequest {
  pub loaders: Vec<String>,
  pub resource: String,
  pub source: String,
  pub context: String,
}

pub type JsLoaderRunner = Arc<dyn Fn(JsLoaderRequest) -> Result<String, String>>;

/// 带路径或 `.js` 后缀的视为 JS loader（npm / 本地 loader 文件）
pub fn is_js_loader(name: &str) -> bool {
  name.contains('/') || name.contains('\\') || name.ends_with(".js")
}

pub fn split_loader_chain(loader_chain: &[String]) -> (Vec<String>, Vec<String>) {
  let mut rust_loaders = Vec::new();
  let mut js_loaders = Vec::new();

  for name in loader_chain {
    if is_js_loader(name) {
      js_loaders.push(name.clone());
    } else {
      rust_loaders.push(name.clone());
    }
  }

  (rust_loaders, js_loaders)
}
