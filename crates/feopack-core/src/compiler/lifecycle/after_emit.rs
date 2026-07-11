use super::super::Compiler;

impl Compiler {
  pub(crate) fn after_emit(&self) {
    println!("[rust compiler lifecycle] after_emit");
  }
}
