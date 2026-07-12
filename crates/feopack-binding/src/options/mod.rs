use napi_derive::napi;

#[derive(Debug, Clone)]
#[napi(object, object_to_js = false)]
pub struct Output {
  pub path: String,
  pub filename: String,
}

#[derive(Debug, Clone)]
#[napi(object, object_to_js = false)]
pub struct ModuleRule {
  pub test: String,
  pub use_loaders: Vec<String>,
}

#[derive(Debug, Clone)]
#[napi(object, object_to_js = false)]
pub struct ModuleConfig {
  pub rules: Vec<ModuleRule>,
}

/** 直接映射前端的打包配置文件 */
#[derive(Debug, Clone)]
#[napi(object, object_to_js = false)]
pub struct RawOptions {
  pub entry: String,
  pub mode: String,
  pub context: String,
  pub output: Output,
  pub module: Option<ModuleConfig>,
  pub rust_plugins: Option<Vec<String>>,
}
