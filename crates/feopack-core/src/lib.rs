mod compiler;
mod module_graph;
mod utils;

// 踩坑：* 不会导出子模块的内容
pub use compiler::compilation::{CompilationOptions, Output};
pub use compiler::*;
pub use module_graph::*;
