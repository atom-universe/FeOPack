use super::super::Compiler;

impl Compiler {
  pub(crate) fn before_run(&self) -> Result<(), String> {
    println!("[rust compiler lifecycle] before_run");
    self.hooks.before_run.call(&())
  }
}
