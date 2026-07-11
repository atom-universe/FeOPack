use super::super::Compiler;

impl Compiler {
  pub(crate) fn compile_lifecycle(&self) -> Result<(), String> {
    println!("[rust compiler lifecycle] compile");
    self.hooks.compile.call()
  }
}
