use crate::compilation::ResolvedModule;
use crate::loader::inline_request::{self};

use super::{Compilation, ResolvedPath};
use std::path::{Path, PathBuf};

impl Compilation {
  pub(crate) fn create_module_id(path: &PathBuf) -> Result<String, String> {
    Ok(Self::normalize_path(path)?.to_string_lossy().to_string())
  }

  // path.join 之后会留下 ./ 这类 CurDir 组件。
  // module id 需要稳定一点，所以这里先做最小 normalize。
  pub(crate) fn normalize_path(path: &PathBuf) -> Result<PathBuf, String> {
    let normalized: PathBuf = path
      .components()
      .filter(|c| !matches!(c, std::path::Component::CurDir))
      .collect();
    Ok(normalized)
  }

  pub(crate) fn resolve_path(
    dep: &str,
    module_dir: &Path,
    _context: &Path,
  ) -> Result<ResolvedPath, String> {
    let inline = inline_request::parse_inline_request(dep);
    let resource = inline.resource.as_str();

    // 当前边界：非相对路径都当 external。
    if Self::is_external_request(resource) {
      return Ok(ResolvedPath::External(dep.to_string()));
    }

    let (request_path, query) = resource
      .split_once('?')
      .map(|(p, q)| (p, format!("?{}", q)))
      .unwrap_or((resource, String::new()));
    println!("========\nrequest_path: {:?} \nquery: {:?}\n", request_path, query);

    let dep_path = module_dir.join(request_path);
    let normalized = Self::normalize_path(&dep_path)?;

    let resource_path = if !normalized.exists() && normalized.extension().is_none() {
      let dep_path_with_js = normalized.with_extension("js");
      if dep_path_with_js.exists() {
        dep_path_with_js
      } else {
        normalized
      }
    } else {
      normalized
    };

    let resource_id = Self::create_module_id(&resource_path)?;
    let module_id = inline_request::build_module_id(&inline, &resource_id, &query);

    Ok(ResolvedPath::File(ResolvedModule {
      module_id,
      resource_path,
      resource_query: query,
      inline,
    }))
  }

  pub fn is_external_request(request: &str) -> bool {
    !request.starts_with('.') && !request.starts_with('/')
  }
}
