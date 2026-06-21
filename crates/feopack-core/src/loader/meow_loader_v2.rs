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

pub fn meow_loader_v2(context: LoaderContext) -> Result<String, String> {
  let file_name = context
    .resource_path
    .file_name()
    .and_then(|name| name.to_str())
    .ok_or_else(|| "invalid resource path".to_string())?;

  println!("\n========\ncontext: {:?}\n", context);

  let blocks = detect_blocks(&context.source);
  println!("========\nblocks: {:?}\n", blocks);
  let virtual_imports = generate_virtual_import_lines(file_name, &blocks);
  println!("========\nvirtual_imports: {:?}\n", virtual_imports);
  // 暂时先原样吐出 virtual import，下一步再接 resolver + 子 loader
  Ok(virtual_imports.join("\n"))
}
