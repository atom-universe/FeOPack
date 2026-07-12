use std::path::PathBuf;

use crate::loader::inline_request;
use crate::loader::LoaderRegistry;

#[derive(Debug, Clone)]
pub(crate) struct NormalModuleCreateData {
  pub(crate) module_id: String,
  pub(crate) resource_path: PathBuf,
  pub(crate) resource_query: String,
  pub(crate) loaders: Vec<String>,
}

#[derive(Debug, Default)]
pub(crate) struct NormalModuleFactory;

impl NormalModuleFactory {
  pub(crate) fn new() -> Self {
    Self
  }

  pub(crate) fn create(
    &self,
    module_id: String,
    loader_registry: &LoaderRegistry,
  ) -> NormalModuleCreateData {
    let inline = inline_request::parse_inline_request(&module_id);
    let (module_resource_path, resource_query) = inline
      .resource
      .split_once('?')
      .map(|(p, q)| (p, format!("?{}", q)))
      .unwrap_or_else(|| (inline.resource.as_str(), String::new()));

    let resource_path = PathBuf::from(module_resource_path);
    let loaders = loader_registry.resolve_chain(&resource_path, &resource_query, &inline);

    NormalModuleCreateData {
      module_id,
      resource_path,
      resource_query,
      loaders,
    }
  }
}
