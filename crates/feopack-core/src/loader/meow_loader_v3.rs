use crate::loader::meow_loader_v2::{detect_blocks, MeowBlock};
use crate::loader::{LoaderContext, PitchResult};

/// v3 pitcher：读盘前根据 resource_query 认路。
///
/// 与 v2 的区别：v2 把子 block 配方写在 inline import 里；v3 用 query + config rule，
/// pitcher 在 pitch 阶段先跑，避免子 request 误命中主 SFC rule。
pub fn meow_v3_pitch(context: &LoaderContext) -> Result<PitchResult, String> {
  let query = context.resource_query.as_str();

  if query.is_empty() {
    return Ok(PitchResult::Continue);
  }

  if !query.starts_with("?type=") {
    return Err(format!("meow-v3-pitcher: 无法识别的 query: {query}"));
  }

  Ok(PitchResult::Continue)
}

pub fn generate_virtual_import_lines_v3(file_name: &str, blocks: &[MeowBlock]) -> Vec<String> {
  blocks
    .iter()
    .map(|block| {
      format!(
        "import __{}__ from './{file_name}{}';",
        block.block_type,
        block.build_query()
      )
    })
    .collect()
}

/// 主请求：只生成 plain virtual import（无 inline loader 链）
pub fn meow_loader_v3_main(context: LoaderContext) -> Result<String, String> {
  let file_name = context
    .resource_path
    .file_name()
    .and_then(|name| name.to_str())
    .ok_or_else(|| "invalid resource path".to_string())?;

  let mut blocks = detect_blocks(&context.source);
  blocks.sort_by_key(|block| match block.block_type {
    "style" => 0,
    "template" => 1,
    "script" => 2,
    _ => 3,
  });
  let virtual_imports = generate_virtual_import_lines_v3(file_name, &blocks);

  Ok(format!(
    r#"{imports}
function meow() {{
  {calls}
}}
export {{ meow as default }};"#,
    imports = virtual_imports.join("\n"),
    calls = blocks
      .iter()
      .map(|block| format!("__{}__();", block.block_type))
      .collect::<Vec<_>>()
      .join("\n  ")
  ))
}

pub fn meow_v3_pitcher_normal(context: LoaderContext) -> Result<String, String> {
  Ok(context.source)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

  #[test]
  fn v3_virtual_imports_do_not_use_inline_loaders() {
    let blocks = detect_blocks(
      r#"<meow>hi</meow>
<script lang="ts">console.log(1)</script>
<style scoped>.a{}</style>"#,
    );
    let imports = generate_virtual_import_lines_v3("index.meow-v3", &blocks);

    assert_eq!(imports.len(), 3);
    for line in imports {
      assert!(!line.contains("-!"));
      assert!(!line.contains("meow-extract"));
      assert!(line.contains("./index.meow-v3?"));
    }
  }

  #[test]
  fn v3_pitcher_accepts_block_queries() {
    let ctx = LoaderContext {
      resource_path: PathBuf::from("./index.meow-v3"),
      resource_query: "?type=script&lang=ts".to_string(),
      source: String::new(),
    };

    assert_eq!(meow_v3_pitch(&ctx).unwrap(), PitchResult::Continue);
  }

  #[test]
  fn v3_pitcher_rejects_unknown_query() {
    let ctx = LoaderContext {
      resource_path: PathBuf::from("./index.meow-v3"),
      resource_query: "?weird=1".to_string(),
      source: String::new(),
    };

    assert!(meow_v3_pitch(&ctx).is_err());
  }
}
