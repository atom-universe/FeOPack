use crate::loader::LoaderContext;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MeowBlock {
  pub block_type: &'static str,
  pub attrs: HashMap<String, String>,
}

impl MeowBlock {
  pub fn build_query(&self) -> String {
    let mut parts = vec![format!("type={}", self.block_type)];

    match self.block_type {
      "script" => {
        let lang = self.attrs.get("lang").map(|s| s.as_str()).unwrap_or("js");
        parts.push(format!("lang={lang}"));
      }
      "style" if self.attrs.contains_key("scoped") => {
        parts.push("scoped".to_string());
      }
      _ => {}
    }

    format!("?{}", parts.join("&"))
  }
}

pub fn parse_tag_attrs(attr_str: &str) -> HashMap<String, String> {
  let mut attrs = HashMap::new();
  let mut rest = attr_str.trim();

  while !rest.is_empty() {
    if let Some(stripped) = rest.strip_prefix("scoped") {
      attrs.insert("scoped".to_string(), String::new());
      rest = stripped.trim();
      continue;
    }

    let Some(eq_index) = rest.find('=') else {
      break;
    };

    let name = rest[..eq_index].trim();
    rest = rest[eq_index + 1..].trim();

    let (value, remaining) = if rest.starts_with('"') {
      let end = rest[1..]
        .find('"')
        .map(|index| index + 2)
        .unwrap_or(rest.len());
      (rest[1..end - 1].to_string(), rest[end..].trim())
    } else if rest.starts_with('\'') {
      let end = rest[1..]
        .find('\'')
        .map(|index| index + 2)
        .unwrap_or(rest.len());
      (rest[1..end - 1].to_string(), rest[end..].trim())
    } else {
      let end = rest
        .find(|c: char| c.is_whitespace())
        .unwrap_or(rest.len());
      (rest[..end].to_string(), rest[end..].trim())
    };

    attrs.insert(name.to_string(), value);
    rest = remaining;
  }

  attrs
}

fn find_opening_tag(source: &str, tag_name: &str) -> Option<(HashMap<String, String>, usize, usize)> {
  let start = source.find(&format!("<{tag_name}"))?;
  let tag_end = source[start..].find('>')? + start;
  let attr_str = &source[start + tag_name.len() + 1..tag_end];
  let content_start = tag_end + 1;
  let close_tag = format!("</{tag_name}>");
  let content_end = source.find(&close_tag)?;
  Some((parse_tag_attrs(attr_str), content_start, content_end))
}

/// 检测 .meow-v2 里有哪些块及其属性
/// （另外，才发现 rust 的 doc 语法竟然是这样的
pub fn detect_blocks(source: &str) -> Vec<MeowBlock> {
  let mut blocks = Vec::new();

  if source.contains("<meow>") {
    blocks.push(MeowBlock {
      block_type: "template",
      attrs: HashMap::new(),
    });
  }

  if source.contains("<script") {
    if let Some((attrs, _, _)) = find_opening_tag(source, "script") {
      blocks.push(MeowBlock {
        block_type: "script",
        attrs,
      });
    }
  }

  if source.contains("<style") {
    if let Some((attrs, _, _)) = find_opening_tag(source, "style") {
      blocks.push(MeowBlock {
        block_type: "style",
        attrs,
      });
    }
  }

  blocks
}

/// 为每个块生成带 inline loader 的 import 行
/// 
pub fn block_inline_loaders(block: &MeowBlock) -> &'static str {
  match block.block_type {
    // type=script&lang=ts
    "script" if block.attrs.get("lang").map(|s| s.as_str()) == Some("ts") => {
      "meow-wrap-script-export!typescript-loader!meow-extract-script"
    }
    // type=script&lang=js
    "script" => "meow-wrap-script-export!meow-extract-script",
    // type=template
    "template" => "meow-wrap-template-export!meow-extract-template",
    // type=style&scoped
    "style" if block.attrs.contains_key("scoped") => {
      "meow-wrap-style-export!meow-scope-style!meow-extract-style"
    }
    // type=style
    "style" => "meow-wrap-style-export!meow-extract-style",
    _ => "",
  }
}

pub fn generate_virtual_import_lines(file_name: &str, blocks: &[MeowBlock]) -> Vec<String> {
  blocks
    .iter()
    .map(|block| {
      format!(
        "import __{}__ from '-!{}!./{file_name}{}';",
        block.block_type,
        block_inline_loaders(block),
        block.build_query()
      )
    })
    .collect()
}

pub fn extract_template(source: &str) -> Result<String, String> {
  let start = source.find("<meow>").ok_or_else(|| "missing <meow> tag".to_string())?;
  let end = source
    .find("</meow>")
    .ok_or_else(|| "missing </meow> tag".to_string())?;
  Ok(source[start + "<meow>".len()..end].trim().to_string())
}

pub fn extract_script(source: &str) -> Result<String, String> {
  let (_, content_start, content_end) =
    find_opening_tag(source, "script").ok_or_else(|| "missing <script> tag".to_string())?;
  Ok(source[content_start..content_end].trim().to_string())
}

pub fn extract_style(source: &str) -> Result<String, String> {
  let (_, content_start, content_end) =
    find_opening_tag(source, "style").ok_or_else(|| "missing <style> tag".to_string())?;
  Ok(source[content_start..content_end].trim().to_string())
}

pub fn scope_css(css: &str, scope_id: &str) -> String {
  css.split('}')
    .filter(|part| !part.trim().is_empty())
    .map(|part| {
      let Some((selector, body)) = part.split_once('{') else {
        return part.to_string();
      };

      let selector = selector.trim();
      if selector.starts_with('@') {
        format!("{selector}{{{body}}}")
      } else {
        format!("#{scope_id} {selector} {{{body}}}")
      }
    })
    .collect::<Vec<_>>()
    .join("\n")
}

/// 主请求：生成 virtual import 并组装 default export
pub fn meow_loader_v2_main(context: LoaderContext) -> Result<String, String> {
  let file_name = context
    .resource_path
    .file_name()
    .and_then(|name| name.to_str())
    .ok_or_else(|| "invalid resource path".to_string())?;

  let mut blocks = detect_blocks(&context.source);
  // 一个小细节就是，这里得先处理好 style，再去处理 template
  // 否则后续做热更新的时候就会有一些问题
  blocks.sort_by_key(|block| match block.block_type {
    "style" => 0,
    "template" => 1,
    "script" => 2,
    _ => 3,
  });
  let virtual_imports = generate_virtual_import_lines(file_name, &blocks);

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

pub fn meow_extract_template(context: LoaderContext) -> Result<String, String> {
  extract_template(&context.source)
}

pub fn meow_extract_script(context: LoaderContext) -> Result<String, String> {
  extract_script(&context.source)
}

pub fn meow_extract_style(context: LoaderContext) -> Result<String, String> {
  extract_style(&context.source)
}

pub fn meow_scope_style(context: LoaderContext) -> Result<String, String> {
  Ok(scope_css(context.source.trim(), "meow"))
}

pub fn meow_wrap_template_export(context: LoaderContext) -> Result<String, String> {
  Ok(format!(
    r#"function __meow_template__() {{
  const element = document.getElementById('meow');
  if (element) {{
    element.innerHTML = {html:?};
  }}
}}
export {{ __meow_template__ as default }};"#,
    html = context.source.trim()
  ))
}

pub fn meow_wrap_script_export(context: LoaderContext) -> Result<String, String> {
  Ok(format!(
    "function __meow_script__() {{\n{}\n}}\nexport {{ __meow_script__ as default }};",
    context.source.trim()
  ))
}

pub fn meow_wrap_style_export(context: LoaderContext) -> Result<String, String> {
  Ok(format!(
    r#"function __meow_style__() {{
  if (typeof document === 'undefined') return;
  const style = document.createElement('style');
  style.textContent = {css:?};
  document.head.appendChild(style);
}}
export {{ __meow_style__ as default }};"#,
    css = context.source.trim()
  ))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_script_lang_attr() {
    let attrs = parse_tag_attrs(r#" lang="ts" "#);
    assert_eq!(attrs.get("lang").map(String::as_str), Some("ts"));
  }

  #[test]
  fn build_script_query_with_lang() {
    let block = MeowBlock {
      block_type: "script",
      attrs: HashMap::from([("lang".to_string(), "ts".to_string())]),
    };
    assert_eq!(block.build_query(), "?type=script&lang=ts");
  }

  #[test]
  fn build_style_query_with_scoped() {
    let block = MeowBlock {
      block_type: "style",
      attrs: HashMap::from([("scoped".to_string(), String::new())]),
    };
    assert_eq!(block.build_query(), "?type=style&scoped");
  }
}
