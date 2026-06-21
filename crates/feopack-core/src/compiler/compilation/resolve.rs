use crate::compilation::ResolvedModule;

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

  // TODO: 在这里处理 query
  pub(crate) fn resolve_path(
    dep: &str,
    module_dir: &Path,
    _context: &Path,
  ) -> Result<ResolvedPath, String> {
    // 当前边界：非相对路径都当 external。
    // 也就是 import fs from 'node:fs' / import lodash from 'lodash' 会在 runtime require。
    if Self::is_external_request(dep) {
      return Ok(ResolvedPath::External(dep.to_string()));
    }

    // 拆分下虚拟请求
    let (request_path, query) = dep
    .split_once('?')
    .map(|(p, q)| (p, format!("?{}", q)))
    .unwrap_or((dep, String::new()));
    println!("========\nrequest_path: {:?} \nquery: {:?}\n", request_path, query);
    // 如果依赖路径以 . 或 .. 开头，相对于当前模块目录解析。
    let dep_path = module_dir.join(request_path);
    let normalized = Self::normalize_path(&dep_path)?;
 
    // 处理扩展名，如果没有扩展名，就用 .js
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

    Ok(ResolvedPath::File(ResolvedModule {
      module_id: format!("{}{}", Self::create_module_id(&resource_path)?, query),
      resource_path,
      resource_query: query,
    }))
  }

  pub fn is_external_request(request: &str) -> bool {
    !request.starts_with('.')
  }
}
