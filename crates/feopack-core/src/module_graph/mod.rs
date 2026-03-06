pub mod module;
pub use module::Module;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModuleGraphPartial {
  pub modules: HashMap<String, Module>,
}

#[derive(Debug, Default)]
pub struct ModuleGraph {
  // 已构建好的内容，只读
  pub(crate) partials: Vec<ModuleGraphPartial>,
  // 增量构建可编辑的部分，可写
  // active: Option<ModuleGraphPartial>,
}

impl ModuleGraph {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn add_module(&mut self, partial: ModuleGraphPartial) {
    self.partials.push(partial);
  }

  pub fn add_single_module(&mut self, module_id: String, module: Module) {
    // 查找最后一个 partial，如果为空则创建新的
    if self.partials.is_empty() {
      self.partials.push(ModuleGraphPartial {
        modules: std::collections::HashMap::new(),
      });
    }

    // 添加到最后一个 partial
    if let Some(last) = self.partials.last_mut() {
      last.modules.insert(module_id, module);
    }
  }

  pub fn has_module(&self, id: &str) -> bool {
    for partial in &self.partials {
      if partial.modules.contains_key(id) {
        return true;
      }
    }
    false
  }

  pub fn get_module(&self, id: &str) -> Option<Module> {
    for partial in &self.partials {
      if let Some(module) = partial.modules.get(id) {
        return Some(module.clone());
      }
    }
    None
  }
}
