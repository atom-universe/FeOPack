use crate::loader::LoaderContext;

/// 从任意源码里找出 import 行（以后 <script> 块里有 import 时用）
pub fn get_import_lines(source: &str) -> Vec<String> {
  source
    .lines()
    .map(|line| line.trim())
    .filter(|line| line.starts_with("import "))
    .map(|line| line.to_string())
    .collect()
}

/// 检测 .meow-v2 里有哪些块
pub fn detect_blocks(source: &str) -> Vec<&'static str> {
  let mut blocks = Vec::new();
  if source.contains("<meow>") {
    blocks.push("template");
  }
  if source.contains("<script") {
    blocks.push("script");
  }
  blocks
}

/// virtual request 第一步：为每个块生成 import 行
pub fn generate_virtual_import_lines(file_name: &str, blocks: &[&str]) -> Vec<String> {
  blocks
    .iter()
    .map(|block_type| {
      format!("import __{block_type}__ from './{file_name}?type={block_type}';")
    })
    .collect()
}

// 拆分1
pub fn extract_template(source: &str) -> Result<String, String> {
  let start = source.find("<meow>").ok_or_else(|| "missing <meow> tag".to_string())?;
  let end = source
    .find("</meow>")
    .ok_or_else(|| "missing </meow> tag".to_string())?;
  Ok(source[start + "<meow>".len()..end].trim().to_string())
}

// 拆分2
pub fn extract_script(source: &str) -> Result<String, String> {
  let start_tag = source
    .find("<script")
    .ok_or_else(|| "missing <script> tag".to_string())?;
  // TODO: 兼容下一些标签属性
  let content_start = source[start_tag..]
    .find('>')
    .map(|index| start_tag + index + 1)
    .ok_or_else(|| "malformed <script> tag".to_string())?;
  let end = source
    .find("</script>")
    .ok_or_else(|| "missing </script> tag".to_string())?;
  Ok(source[content_start..end].trim().to_string())
}

/// 主请求：生成 virtual import 并组装 default export
pub fn meow_loader_v2_main(context: LoaderContext) -> Result<String, String> {
  let file_name = context
    .resource_path
    .file_name()
    .and_then(|name| name.to_str())
    .ok_or_else(|| "invalid resource path".to_string())?;

  let blocks = detect_blocks(&context.source);
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
      .map(|block| format!("__{block}__();"))
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
