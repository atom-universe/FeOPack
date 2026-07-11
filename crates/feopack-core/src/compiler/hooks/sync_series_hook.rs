pub(crate) struct SyncSeriesHook {
  name: &'static str,
}

impl SyncSeriesHook {
  pub(crate) fn new(name: &'static str) -> Self {
    Self { name }
  }

  pub(crate) fn call(&self) -> Result<(), String> {
    println!("[rust hook] {}", self.name);
    Ok(())
  }
}
