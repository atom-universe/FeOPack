use super::super::Compiler;

impl Compiler {
  pub(crate) fn run_lifecycle(&self) -> Result<(), String> {
    println!("[rust compiler lifecycle] run");
    self.hooks.run.call()
  }
}
