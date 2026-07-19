use super::hooks::AssetEmittedContext;
use super::Compiler;
use std::path::Path;
use tokio::fs;

impl Compiler {
  pub async fn emit_assets(&mut self) -> Result<(), String> {
    self.hooks().emit.call(&())?;
    self.call_js_compiler_hook("emit", None, None).await?;
    for asset in &self.compilation.assets {
      let output_dir = Path::new(&self.options.output.path);
      let output_file = output_dir.join(&asset.filename);

      if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)
          .await
          .map_err(|e| format!("创建目录失败 {:?}: {}", parent, e))?;
      }

      fs::write(&output_file, asset.source.as_bytes())
        .await
        .map_err(|e| format!("写入失败 {:?}: {}", output_file, e))?;
      self.hooks().asset_emitted.call(&AssetEmittedContext {
        filename: asset.filename.clone(),
        target_path: output_file.clone(),
      })?;
      self
        .call_js_compiler_hook(
          "assetEmitted",
          Some(asset.filename.clone()),
          Some(output_file.to_string_lossy().into_owned()),
        )
        .await?;
    }

    self.hooks().after_emit.call(&())?;
    self.call_js_compiler_hook("afterEmit", None, None).await?;
    Ok(())
  }
}
