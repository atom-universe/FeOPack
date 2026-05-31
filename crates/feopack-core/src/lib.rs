mod compiler;
mod loader;
mod module_graph;
mod swc_compiler;
mod utils;

// 踩坑：* 不会导出子模块的内容
pub use compiler::compilation::{CompilationOptions, Output};
pub use compiler::*;
pub use loader::*;
pub use module_graph::*;
pub use swc_compiler::*;
