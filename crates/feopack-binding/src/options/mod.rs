use napi_derive::napi;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct Output {
  pub path: String,
  pub filename: String,
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawOptions {
  // 前端保证数据一定存在
  pub mode: String,
  pub context: String,
  // entry 走插件处理，不是由 core 直接处理
  // pub entry: String,
  pub output: Output,
}
