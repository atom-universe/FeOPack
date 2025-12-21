use napi_derive::napi;

#[derive(Debug, Clone)]
#[napi(object, object_to_js = false)]
pub struct Output {
  pub path: String,
  pub filename: String,
}

/** 直接映射前端的打包配置文件 */
#[derive(Debug, Clone)]
#[napi(object, object_to_js = false)]
pub struct RawOptions {
  // 前端保证数据一定存在
  pub entry: String,
  pub mode: String,
  pub context: String,
  pub output: Output,
}
