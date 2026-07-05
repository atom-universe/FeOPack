use napi_derive::napi;

#[napi(object)]
#[derive(Clone)]
pub struct JsLoaderContextInput {
  pub loader_state: String,
  pub loaders: Vec<String>,
  pub resource: String,
  pub source: String,
  pub project_root: String,
  pub skip_read_resource: bool,
}

#[napi(object)]
#[derive(Clone)]
pub struct JsLoaderResultOutput {
  pub source: String,
  pub short_circuit: bool,
  pub pitched_loader_index: Option<u32>,
}
