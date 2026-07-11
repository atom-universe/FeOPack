use super::super::Compiler;

impl Compiler {
  pub(crate) fn done(&self) -> Result<(), String> {
    println!("[rust compiler lifecycle] done");
    self.hooks.done.call()
  }
}
