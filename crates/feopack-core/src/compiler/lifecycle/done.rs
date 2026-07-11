use super::super::Compiler;

impl Compiler {
  pub(crate) fn done(&self) {
    println!("[rust compiler lifecycle] done");
  }
}
