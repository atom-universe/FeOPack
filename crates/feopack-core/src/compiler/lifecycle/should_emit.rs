use super::super::Compiler;

impl Compiler {
  pub(crate) fn should_emit(&self) -> Result<bool, String> {
    println!("[rust compiler lifecycle] should_emit");
    Ok(self.hooks.should_emit.call(&())?.unwrap_or(true))
  }
}
