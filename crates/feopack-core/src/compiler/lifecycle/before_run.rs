use super::super::Compiler;

impl Compiler {
  pub(crate) fn before_run(&self) {
    println!("[rust compiler lifecycle] before_run");
  }
}
