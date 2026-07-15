use super::super::Compiler;

impl Compiler {
  pub(crate) fn after_emit(&self) -> Result<(), String> {
    println!("[rust compiler lifecycle] after_emit");
    self.hooks().after_emit.call(&())
  }
}
