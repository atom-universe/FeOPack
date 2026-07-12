use super::super::Compiler;
use crate::compiler::compilation::GeneratedAsset;
use crate::compiler::hooks::AssetEmittedContext;
use std::path::Path;

impl Compiler {
  pub(crate) fn asset_emitted(
    &self,
    asset: &GeneratedAsset,
    target_path: &Path,
  ) -> Result<(), String> {
    println!(
      "[rust compiler lifecycle] asset_emitted filename={} target={}",
      asset.filename,
      target_path.display()
    );
    self.hooks.asset_emitted.call(&AssetEmittedContext {
      filename: asset.filename.clone(),
      target_path: target_path.to_path_buf(),
    })
  }
}
