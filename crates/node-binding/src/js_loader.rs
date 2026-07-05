use crate::js_loader_types::{JsLoaderContextInput, JsLoaderResultOutput};
use feopack_core::JsLoaderRequest;
use napi::bindgen_prelude::*;
use std::sync::Arc;

use feopack_core::JsLoaderRunner;

pub fn create_js_loader_runner(
  js_runner: Arc<FunctionRef<JsLoaderContextInput, JsLoaderResultOutput>>,
  env: Env,
) -> JsLoaderRunner {
  Arc::new(move |req: JsLoaderRequest| {
    let js_fn = js_runner
      .borrow_back(&env)
      .map_err(|e| e.to_string())?;
    let output = js_fn
      .call(JsLoaderContextInput {
        loader_state: "normal".to_string(),
        loaders: req.loaders,
        resource: req.resource,
        source: req.source,
        context: req.context,
        skip_read_resource: true,
      })
      .map_err(|e| e.to_string())?;
    Ok(output.source)
  })
}
