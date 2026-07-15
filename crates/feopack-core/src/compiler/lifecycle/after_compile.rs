use super::super::Compiler;

impl Compiler {
  pub(crate) fn after_compile(&self) -> Result<(), String> {
    println!("[rust compiler lifecycle] after_compile");
    self.hooks().after_compile.call(&())
  }
}
