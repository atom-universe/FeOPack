pub mod module;
use module::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModuleGraphPartial {
  pub modules: HashMap<String, Module>,
}

#[derive(Debug, Default)]
pub struct ModuleGraph {
  // 已构建好的内容，只读
  partials: Vec<ModuleGraphPartial>,
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
