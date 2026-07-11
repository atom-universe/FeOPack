use super::super::Compiler;

impl Compiler {
  pub(crate) fn before_compile(&self) {
    println!("[rust compiler lifecycle] before_compile");
  }
}
