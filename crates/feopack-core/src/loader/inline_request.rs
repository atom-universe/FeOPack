#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InlineRequest {
  /// `-!` / `!!` 前缀：只跑 inline loader，跳过 rule 匹配
  pub inline_only: bool,
  pub loaders: Vec<String>,
  pub resource: String,
}

/// 解析 webpack 风格 inline loader 请求。
///
/// 例：
/// - `./foo.ts`
/// - `typescript-loader!./foo.ts`
/// - `-!meow-wrap-script-export!typescript-loader!meow-extract-script!./foo.meow-v2?type=script&lang=ts`
pub fn parse_inline_request(request: &str) -> InlineRequest {
  let mut rest = request.trim();
  let mut inline_only = false;

  loop {
    if let Some(stripped) = rest.strip_prefix("-!") {
      inline_only = true;
      rest = stripped;
      continue;
    }
    if let Some(stripped) = rest.strip_prefix("!!") {
      inline_only = true;
      rest = stripped;
      continue;
    }
    if let Some(stripped) = rest.strip_prefix('!') {
      inline_only = true;
      rest = stripped;
      continue;
    }
    break;
  }

  if !rest.contains('!') {
    return InlineRequest {
      inline_only,
      loaders: Vec::new(),
      resource: rest.to_string(),
    };
  }

  let parts: Vec<&str> = rest.split('!').collect();
  let resource_index = parts
    .iter()
    .rposition(|part| part.starts_with('.') || part.starts_with('/'))
    .unwrap_or(parts.len() - 1);

  if resource_index == 0 && !parts[0].starts_with('.') && !parts[0].starts_with('/') {
    return InlineRequest {
      inline_only,
      loaders: Vec::new(),
      resource: rest.to_string(),
    };
  }

  InlineRequest {
    inline_only,
    loaders: parts[..resource_index]
      .iter()
      .map(|loader| loader.to_string())
      .collect(),
    resource: parts[resource_index..].join("!"),
  }
}

pub fn build_module_id(
  inline: &InlineRequest,
  resource_path: &str,
  query: &str,
) -> String {
  let base = format!("{resource_path}{query}");
  if inline.loaders.is_empty() {
    base
  } else if inline.inline_only {
    format!("-!{}!{base}", inline.loaders.join("!"))
  } else {
    format!("{}!{base}", inline.loaders.join("!"))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_plain_resource() {
    let parsed = parse_inline_request("./index.meow-v2?type=template");
    assert_eq!(
      parsed,
      InlineRequest {
        inline_only: false,
        loaders: vec![],
        resource: "./index.meow-v2?type=template".to_string(),
      }
    );
  }

  #[test]
  fn parse_inline_loaders_with_dash_bang() {
    let parsed = parse_inline_request(
      "-!meow-wrap-script-export!typescript-loader!meow-extract-script!./index.meow-v2?type=script&lang=ts",
    );
    assert!(parsed.inline_only);
    assert_eq!(
      parsed.loaders,
      vec![
        "meow-wrap-script-export".to_string(),
        "typescript-loader".to_string(),
        "meow-extract-script".to_string(),
      ]
    );
    assert_eq!(
      parsed.resource,
      "./index.meow-v2?type=script&lang=ts".to_string()
    );
  }
}
