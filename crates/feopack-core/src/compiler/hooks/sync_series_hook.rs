type TapFn<Ctx> = Box<dyn Fn(&Ctx) -> Result<(), String> + Send + Sync>;

pub(crate) struct SyncSeriesHook<Ctx> {
  name: &'static str,
  taps: Vec<Tap<Ctx>>,
}

struct Tap<Ctx> {
  name: &'static str,
  function: TapFn<Ctx>,
}

impl<Ctx> SyncSeriesHook<Ctx> {
  pub(crate) fn new(name: &'static str) -> Self {
    Self {
      name,
      taps: Vec::new(),
    }
  }

  #[allow(dead_code)]
  pub(crate) fn tap<F>(&mut self, name: &'static str, function: F)
  where
    F: Fn(&Ctx) -> Result<(), String> + Send + Sync + 'static,
  {
    self.taps.push(Tap {
      name,
      function: Box::new(function),
    });
  }

  pub(crate) fn call(&self, ctx: &Ctx) -> Result<(), String> {
    println!("[rust hook] {}", self.name);
    for tap in &self.taps {
      (tap.function)(ctx).map_err(|e| format!("{} tap {} failed: {}", self.name, tap.name, e))?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::SyncSeriesHook;
  use std::sync::{Arc, Mutex};

  #[test]
  fn calls_taps_in_registration_order() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut hook = SyncSeriesHook::<()>::new("test");

    let first_calls = Arc::clone(&calls);
    hook.tap("first", move |_| {
      first_calls.lock().expect("lock calls").push("first");
      Ok(())
    });

    let second_calls = Arc::clone(&calls);
    hook.tap("second", move |_| {
      second_calls.lock().expect("lock calls").push("second");
      Ok(())
    });

    hook.call(&()).expect("hook should pass");

    assert_eq!(*calls.lock().expect("lock calls"), vec!["first", "second"]);
  }

  #[test]
  fn stops_when_a_tap_returns_error() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut hook = SyncSeriesHook::<()>::new("test");

    let first_calls = Arc::clone(&calls);
    hook.tap("first", move |_| {
      first_calls.lock().expect("lock calls").push("first");
      Err("boom".to_string())
    });

    let second_calls = Arc::clone(&calls);
    hook.tap("second", move |_| {
      second_calls.lock().expect("lock calls").push("second");
      Ok(())
    });

    let err = hook.call(&()).expect_err("hook should fail");

    assert!(err.contains("test tap first failed: boom"));
    assert_eq!(*calls.lock().expect("lock calls"), vec!["first"]);
  }
}
