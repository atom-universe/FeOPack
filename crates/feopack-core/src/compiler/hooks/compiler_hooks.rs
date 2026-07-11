use super::SyncSeriesHook;

pub(crate) struct CompilerHooks {
  pub(crate) initialize: SyncSeriesHook,
  pub(crate) before_run: SyncSeriesHook,
  pub(crate) run: SyncSeriesHook,
  pub(crate) before_compile: SyncSeriesHook,
  pub(crate) compile: SyncSeriesHook,
  pub(crate) after_compile: SyncSeriesHook,
  pub(crate) emit: SyncSeriesHook,
  pub(crate) asset_emitted: SyncSeriesHook,
  pub(crate) after_emit: SyncSeriesHook,
  pub(crate) done: SyncSeriesHook,
}

impl Default for CompilerHooks {
  fn default() -> Self {
    Self {
      initialize: SyncSeriesHook::new("compiler.initialize"),
      before_run: SyncSeriesHook::new("compiler.before_run"),
      run: SyncSeriesHook::new("compiler.run"),
      before_compile: SyncSeriesHook::new("compiler.before_compile"),
      compile: SyncSeriesHook::new("compiler.compile"),
      after_compile: SyncSeriesHook::new("compiler.after_compile"),
      emit: SyncSeriesHook::new("compiler.emit"),
      asset_emitted: SyncSeriesHook::new("compiler.asset_emitted"),
      after_emit: SyncSeriesHook::new("compiler.after_emit"),
      done: SyncSeriesHook::new("compiler.done"),
    }
  }
}
