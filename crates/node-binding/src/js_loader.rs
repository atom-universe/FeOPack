use crate::js_loader_types::{JsLoaderContextInput, JsLoaderResultOutput};
use feopack_core::{JsLoaderRequest, JsLoaderRunResult};
use napi::{bindgen_prelude::*, threadsafe_function::ThreadsafeFunction, Status};
use std::sync::Arc;

use feopack_core::JsLoaderRunner;

pub type JsLoaderRunnerTsFn = ThreadsafeFunction<
  JsLoaderContextInput,
  Promise<JsLoaderResultOutput>,
  JsLoaderContextInput,
  Status,
  false,
  false,
  0,
>;

pub fn create_js_loader_runner(js_loader_runner: Arc<JsLoaderRunnerTsFn>) -> JsLoaderRunner {
  Arc::new(move |req: JsLoaderRequest| {
    let js_loader_runner = Arc::clone(&js_loader_runner);

    Box::pin(async move {
      let output = js_loader_runner
        .call_async(JsLoaderContextInput {
          // normal | pitch
          loader_state: req.loader_state,
          loaders: req.loaders,
          resource: req.resource,
          source: req.source,
          project_root: req.project_root,
          skip_read_resource: true,
        })
        .await
        .map_err(|e| e.to_string())?
        .await
        .map_err(|e| e.to_string())?;

      Ok(JsLoaderRunResult {
        source: output.source,
        short_circuit: output.short_circuit,
        pitched_loader_index: output.pitched_loader_index.map(|v| v as usize),
      })
    })
  })
}
