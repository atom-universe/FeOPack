use std::{future::Future, pin::Pin, sync::Arc};

/// Rust → Node 执行 JS loader 链时的请求体
#[derive(Debug, Clone)]
pub struct JsLoaderRequest {
  pub loader_state: String,
  pub loaders: Vec<String>,
  pub resource: String,
  pub source: String,
  pub project_root: String,
}

#[derive(Debug, Clone)]
pub struct JsLoaderRunResult {
  pub source: String,
  pub short_circuit: bool,
  pub pitched_loader_index: Option<usize>,
}

pub type JsLoaderFuture = Pin<Box<dyn Future<Output = Result<JsLoaderRunResult, String>> + Send>>;
pub type JsLoaderRunner = Arc<dyn Fn(JsLoaderRequest) -> JsLoaderFuture + Send + Sync>;

/// 带路径或 `.js` 后缀的视为 JS loader（npm / 本地 loader 文件）
pub fn is_js_loader(name: &str) -> bool {
  name.contains('/') || name.contains('\\') || name.ends_with(".js")
}
