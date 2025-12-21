#[derive(Debug, Clone)]
pub struct Module {
  pub id: String,
  // !虽然最好不要这么存，否则做增量构建之类的会非常麻烦，但是先这么处理。
  // !理想情况是直接存依赖的引用
  pub dependencies: Vec<String>,
}

impl Module {
  pub fn new(id: String, deps: Option<Vec<String>>) -> Self {
    Self {
      id,
      dependencies: deps.unwrap_or_default(),
    }
  }
}
