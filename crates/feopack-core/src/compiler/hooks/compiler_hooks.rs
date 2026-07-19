use super::{AssetEmittedContext, SyncBailHook, SyncSeriesHook};

pub(crate) struct CompilerHooks {
  pub(crate) make: SyncSeriesHook<()>,
  pub(crate) should_emit: SyncBailHook<(), bool>,
  pub(crate) emit: SyncSeriesHook<()>,
  pub(crate) asset_emitted: SyncSeriesHook<AssetEmittedContext>,
  pub(crate) after_emit: SyncSeriesHook<()>,
}

impl Default for CompilerHooks {
  fn default() -> Self {
    Self {
      make: SyncSeriesHook::new("compiler.make"),
      should_emit: SyncBailHook::new("compiler.should_emit"),
      emit: SyncSeriesHook::new("compiler.emit"),
      asset_emitted: SyncSeriesHook::new("compiler.asset_emitted"),
      after_emit: SyncSeriesHook::new("compiler.after_emit"),
    }
  }
}
