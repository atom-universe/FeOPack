use super::LoaderEnforce;

/// webpack 风格 inline request 前缀，控制 **config** 里哪些 enforce 档参与。
///
/// - `!`   → 跳过 config normal
/// - `-!`  → 跳过 config pre + normal（post 仍可能参与）
/// - `!!`  → 跳过 config pre + normal + post
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InlineConfigOverride {
  #[default]
  None,
  SkipNormal,
  SkipPreNormal,
  SkipAllConfig,
}

impl InlineConfigOverride {
  pub fn allows_config_rule(self, enforce: LoaderEnforce) -> bool {
    match (self, enforce) {
      (InlineConfigOverride::None, _) => true,
      (InlineConfigOverride::SkipNormal, LoaderEnforce::Normal) => false,
      (InlineConfigOverride::SkipNormal, _) => true,
      (InlineConfigOverride::SkipPreNormal, LoaderEnforce::Post) => true,
      (InlineConfigOverride::SkipPreNormal, _) => false,
      (InlineConfigOverride::SkipAllConfig, _) => false,
    }
  }

  pub fn prefix(self) -> &'static str {
    match self {
      InlineConfigOverride::None => "",
      InlineConfigOverride::SkipNormal => "!",
      InlineConfigOverride::SkipPreNormal => "-!",
      InlineConfigOverride::SkipAllConfig => "!!",
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InlineRequest {
  pub config_override: InlineConfigOverride,
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
  let mut config_override = InlineConfigOverride::None;

  loop {
    if let Some(stripped) = rest.strip_prefix("-!") {
      config_override = InlineConfigOverride::SkipPreNormal;
      rest = stripped;
      continue;
    }
    if let Some(stripped) = rest.strip_prefix("!!") {
      config_override = InlineConfigOverride::SkipAllConfig;
      rest = stripped;
      continue;
    }
    if let Some(stripped) = rest.strip_prefix('!') {
      config_override = InlineConfigOverride::SkipNormal;
      rest = stripped;
      continue;
    }
    break;
  }

  if !rest.contains('!') {
    return InlineRequest {
      config_override,
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
      config_override,
      loaders: Vec::new(),
      resource: rest.to_string(),
    };
  }

  InlineRequest {
    config_override,
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
  let prefix = inline.config_override.prefix();

  if inline.loaders.is_empty() {
    format!("{prefix}{base}")
  } else {
    format!("{prefix}{}!{base}", inline.loaders.join("!"))
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
        config_override: InlineConfigOverride::None,
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
    assert_eq!(parsed.config_override, InlineConfigOverride::SkipPreNormal);
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

  #[test]
  fn parse_bang_skips_only_normal_config() {
    let parsed = parse_inline_request("!style-loader!css-loader!./a.css");
    assert_eq!(parsed.config_override, InlineConfigOverride::SkipNormal);
    assert!(parsed.config_override.allows_config_rule(LoaderEnforce::Pre));
    assert!(!parsed.config_override.allows_config_rule(LoaderEnforce::Normal));
    assert!(parsed.config_override.allows_config_rule(LoaderEnforce::Post));
  }

  #[test]
  fn parse_double_bang_skips_all_config() {
    let parsed = parse_inline_request("!!style-loader!./a.css");
    assert_eq!(parsed.config_override, InlineConfigOverride::SkipAllConfig);
    assert!(!parsed.config_override.allows_config_rule(LoaderEnforce::Post));
  }

  #[test]
  fn build_module_id_preserves_prefix() {
    let inline = parse_inline_request("-!a!b!./f?type=script");
    assert_eq!(
      build_module_id(&inline, "/abs/f", "?type=script"),
      "-!a!b!/abs/f?type=script"
    );
  }
}
