type TapFn<Ctx, Output> = Box<dyn Fn(&Ctx) -> Result<Option<Output>, String> + Send + Sync>;

pub(crate) struct SyncBailHook<Ctx, Output> {
  name: &'static str,
  taps: Vec<Tap<Ctx, Output>>,
}

struct Tap<Ctx, Output> {
  name: &'static str,
  function: TapFn<Ctx, Output>,
}

impl<Ctx, Output> SyncBailHook<Ctx, Output> {
  pub(crate) fn new(name: &'static str) -> Self {
    Self {
      name,
      taps: Vec::new(),
    }
  }

  #[allow(dead_code)]
  pub(crate) fn tap<F>(&mut self, name: &'static str, function: F)
  where
    F: Fn(&Ctx) -> Result<Option<Output>, String> + Send + Sync + 'static,
  {
    self.taps.push(Tap {
      name,
      function: Box::new(function),
    });
  }

  pub(crate) fn call(&self, ctx: &Ctx) -> Result<Option<Output>, String> {
    println!("[rust hook] {}", self.name);
    for tap in &self.taps {
      if let Some(result) =
        (tap.function)(ctx).map_err(|e| format!("{} tap {} failed: {}", self.name, tap.name, e))?
      {
        return Ok(Some(result));
      }
    }
    Ok(None)
  }
}

#[cfg(test)]
mod tests {
  use super::SyncBailHook;
  use std::sync::{Arc, Mutex};

  #[test]
  fn stops_at_first_tap_that_returns_a_value() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut hook = SyncBailHook::<(), bool>::new("test");

    let first_calls = Arc::clone(&calls);
    hook.tap("first", move |_| {
      first_calls.lock().expect("lock calls").push("first");
      Ok(Some(false))
    });

    let second_calls = Arc::clone(&calls);
    hook.tap("second", move |_| {
      second_calls.lock().expect("lock calls").push("second");
      Ok(Some(true))
    });

    let result = hook.call(&()).expect("hook should pass");

    assert_eq!(result, Some(false));
    assert_eq!(*calls.lock().expect("lock calls"), vec!["first"]);
  }

  #[test]
  fn returns_none_when_no_tap_bails() {
    let mut hook = SyncBailHook::<(), bool>::new("test");

    hook.tap("first", |_| Ok(None));
    hook.tap("second", |_| Ok(None));

    assert_eq!(hook.call(&()).expect("hook should pass"), None);
  }
}
