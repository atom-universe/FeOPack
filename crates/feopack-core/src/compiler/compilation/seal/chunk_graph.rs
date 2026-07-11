use super::super::{Chunk, Compilation};

impl Compilation {
  /**
   * 真实情况下会有一些分组策略，但是这里做简化，
   * 将所有模块放到一个 chunk 中
   */
  pub(crate) async fn create_chunk_graph(&mut self) {
    let mut module_ids = Vec::new();
    for partial in &self.module_graph.partials {
      for module_id in partial.modules.keys() {
        module_ids.push(module_id.clone());
      }
    }
    let chunk = Chunk {
      id: "main".to_string(),
      module_ids,
    };
    self.chunk_graph.chunks.push(chunk);

    eprintln!("modules: {:?}", self.module_graph.partials);
  }
}
