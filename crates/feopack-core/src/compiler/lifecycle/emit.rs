use super::super::Compiler;

impl Compiler {
  pub(crate) fn emit(&self) {
    println!("[rust compiler lifecycle] emit");
  }
}
