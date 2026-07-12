use super::super::Compiler;

impl Compiler {
  pub(crate) fn emit(&self) -> Result<(), String> {
    println!("[rust compiler lifecycle] emit");
    self.hooks.emit.call(&())
  }
}
