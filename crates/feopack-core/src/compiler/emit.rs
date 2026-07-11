use super::Compiler;
use std::path::Path;
use tokio::fs;

impl Compiler {
  pub async fn emit_assets(&mut self) -> Result<(), String> {
    self.emit();
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
      self.asset_emitted(asset, &output_file);
    }

    self.after_emit();
    Ok(())
  }
}
