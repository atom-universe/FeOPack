use feopack_core::{JsCompilerHookEvent, JsHooksAdapter};
use napi::{bindgen_prelude::*, threadsafe_function::ThreadsafeFunction, Status};
use napi_derive::napi;
use std::sync::Arc;

#[napi(object)]
pub struct JsCompilerHookEventInput {
  pub name: String,
  pub filename: Option<String>,
  pub target_path: Option<String>,
}

pub type JsHooksAdapterTsFn = ThreadsafeFunction<
  JsCompilerHookEventInput,
  Promise<()>,
  JsCompilerHookEventInput,
  Status,
  false,
  false,
  0,
>;

pub fn create_js_hooks_adapter(js_hooks_adapter: Arc<JsHooksAdapterTsFn>) -> JsHooksAdapter {
  Arc::new(move |event: JsCompilerHookEvent| {
    let js_hooks_adapter = Arc::clone(&js_hooks_adapter);

    Box::pin(async move {
      js_hooks_adapter
        .call_async(JsCompilerHookEventInput {
          name: event.name.to_string(),
          filename: event.filename,
          target_path: event.target_path,
        })
        .await
        .map_err(|e| e.to_string())?
        .await
        .map_err(|e| e.to_string())
    })
  })
}
