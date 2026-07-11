use super::super::Compiler;
use crate::compiler::compilation::GeneratedAsset;
use std::path::Path;

impl Compiler {
  pub(crate) fn asset_emitted(&self, asset: &GeneratedAsset, target_path: &Path) {
    println!(
      "[rust compiler lifecycle] asset_emitted filename={} target={}",
      asset.filename,
      target_path.display()
    );
  }
}
