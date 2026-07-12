use super::super::Compiler;

impl Compiler {
  pub(crate) fn initialize(&self) -> Result<(), String> {
    println!("[rust compiler lifecycle] initialize");
    self.hooks.initialize.call(&())
  }
}
