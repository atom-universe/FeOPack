use crate::loader::meow_loader_v2::{block_loader_names, block_inline_loaders, detect_blocks, MeowBlock};
use crate::loader::{LoaderContext, PitchResult};
use std::collections::HashMap;

const MEOW_V3_PITCHER: &str = "meow-v3-pitcher";
const MEOW_V3_MAIN: &str = "meow-loader-v3-main";

/// 把 `?type=script&lang=ts` 解析成 [`MeowBlock`]（对齐 vue-loader 的 resourceQuery 约定）
pub fn meow_block_from_query(resource_query: &str) -> Result<MeowBlock, String> {
  let query = resource_query
    .strip_prefix('?')
    .filter(|q| !q.is_empty())
    .ok_or_else(|| format!("meow-v3: expected non-empty resource_query, got {resource_query:?}"))?;

  let mut block_type: Option<&str> = None;
  let mut attrs = HashMap::new();

  for part in query.split('&') {
    if part == "scoped" {
      attrs.insert("scoped".to_string(), String::new());
      continue;
    }

    let Some((key, value)) = part.split_once('=') else {
      continue;
    };

    match key {
      "type" => block_type = Some(value),
      "lang" => {
        attrs.insert("lang".to_string(), value.to_string());
      }
      _ => {}
    }
  }

  let block_type = block_type.ok_or_else(|| format!("meow-v3: missing type= in query {resource_query}"))?;
  let block_type = match block_type {
    "script" => "script",
    "template" => "template",
    "style" => "style",
    other => return Err(format!("meow-v3: unknown block type {other}")),
  };

  let block = MeowBlock {
    block_type,
    attrs,
  };

  println!("block: {:?}", block);
// block: MeowBlock { block_type: "style", attrs: {"scoped": ""} }
// block: MeowBlock { block_type: "style", attrs: {"scoped": ""} }
// block: MeowBlock { block_type: "template", attrs: {} }
// block: MeowBlock { block_type: "template", attrs: {} }
// block: MeowBlock { block_type: "script", attrs: {"lang": "ts"} }
// block: MeowBlock { block_type: "script", attrs: {"lang": "ts"} }

  Ok(block)
}

/// vue-loader 风格：config 只配一条 rule；具体 loader 链由 `resource_query` 在 loader 内部决定
pub fn resolve_meow_v3_chain(resource_query: &str) -> Result<Vec<String>, String> {
  let mut chain = vec![MEOW_V3_PITCHER.to_string()];

  if resource_query.is_empty() {
    chain.push(MEOW_V3_MAIN.to_string());
    return Ok(chain);
  }

  let block = meow_block_from_query(resource_query)?;
  if block_inline_loaders(&block).is_empty() {
    return Err(format!(
      "meow-v3: no loader chain for query {resource_query}"
    ));
  }

  chain.extend(
    block_loader_names(&block)
      .into_iter()
      .map(|name| name.to_string()),
  );
  Ok(chain)
}

/// pitch：识别子 block 请求（类似 vue-loader 的 pitcher）；拼链见 [`resolve_meow_v3_chain`]
pub fn meow_v3_pitch(context: &LoaderContext) -> Result<PitchResult, String> {
  if context.resource_query.is_empty() {
    return Ok(PitchResult::Continue);
  }

  meow_block_from_query(&context.resource_query)?;
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

/// Main request: plain virtual imports (no inline chain in the import string).
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
  fn v3_resolve_main_chain() {
    let chain = resolve_meow_v3_chain("").unwrap();
    assert_eq!(chain, vec![MEOW_V3_PITCHER, MEOW_V3_MAIN]);
  }

  #[test]
  fn v3_resolve_script_ts_chain() {
    let chain = resolve_meow_v3_chain("?type=script&lang=ts").unwrap();
    assert_eq!(
      chain,
      vec![
        MEOW_V3_PITCHER,
        "meow-wrap-script-export",
        "typescript-loader",
        "meow-extract-script",
      ]
    );
  }

  #[test]
  fn v3_resolve_style_scoped_chain() {
    let chain = resolve_meow_v3_chain("?type=style&scoped").unwrap();
    assert_eq!(
      chain,
      vec![
        MEOW_V3_PITCHER,
        "meow-wrap-style-export",
        "meow-scope-style",
        "meow-extract-style",
      ]
    );
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
