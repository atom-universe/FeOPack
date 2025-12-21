use std::sync::Arc;
use swc_common::{sync::Lrc, FileName, SourceMap};
use swc_ecma_ast::{EsVersion, Program};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

pub struct SwcCompiler {
  source_map: Arc<SourceMap>,
}

impl SwcCompiler {
  pub fn new() -> Self {
    Self {
      source_map: Arc::new(SourceMap::default()),
    }
  }

  pub fn parse_js(&self, file_path: std::path::PathBuf, source: String) -> Result<Program, String> {
    // 1. 创建 SourceFile（新版本需要 Lrc<FileName>）
    let filename: Lrc<FileName> = FileName::Real(file_path).into();
    let source_file = self.source_map.new_source_file(filename, source);

    // 2. 配置语法（支持 JSX）
    let syntax = Syntax::Es(swc_ecma_parser::EsSyntax {
      jsx: true,
      ..Default::default()
    });

    // 3. 创建 Lexer（词法分析器）
    let lexer = Lexer::new(
      syntax,
      EsVersion::latest(),
      StringInput::from(&*source_file),
      None,
    );

    // 4. 创建 Parser（语法分析器）
    let mut parser = Parser::new_from(lexer);

    // 5. 解析为 AST
    let program = parser
      .parse_module()
      .map_err(|e| format!("解析错误: {:?}", e))?;

    Ok(Program::Module(program))
  }
}
