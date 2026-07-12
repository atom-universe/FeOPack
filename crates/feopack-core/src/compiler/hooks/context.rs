use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct AssetEmittedContext {
  pub(crate) filename: String,
  pub(crate) target_path: PathBuf,
}
