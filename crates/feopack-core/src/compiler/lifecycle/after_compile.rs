use super::super::Compiler;

impl Compiler {
  pub(crate) fn after_compile(&self) {
    println!("[rust compiler lifecycle] after_compile");
  }
}
