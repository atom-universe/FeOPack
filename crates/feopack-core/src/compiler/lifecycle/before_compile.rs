use super::super::Compiler;

impl Compiler {
  pub(crate) fn before_compile(&self) -> Result<(), String> {
    println!("[rust compiler lifecycle] before_compile");
    self.hooks().before_compile.call(&())
  }
}
