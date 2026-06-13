use crate::loader::LoaderContext;
use swc_common::{sync::Lrc, FileName, Globals, Mark, SourceMap, GLOBALS};
use swc_ecma_ast::EsVersion;
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_transforms_base::{fixer::fixer, resolver};
use swc_ecma_transforms_typescript::strip;

pub fn typescript_loader(context: LoaderContext) -> Result<String, String> {
  let source_map = Lrc::new(SourceMap::default());
  let filename: Lrc<FileName> = FileName::Real(context.resource_path).into();
  let source_file = source_map.new_source_file(filename, context.source);

  let syntax = Syntax::Typescript(TsSyntax {
    tsx: false,
    ..Default::default()
  });

  let lexer = Lexer::new(
    syntax,
    EsVersion::latest(),
    StringInput::from(&*source_file),
    None,
  );

  let mut parser = Parser::new_from(lexer);
  let program = parser
    .parse_program()
    .map_err(|e| format!("TypeScript 解析错误: {:?}", e))?;

  let program = GLOBALS.set(&Globals::default(), || {
    let unresolved_mark = Mark::new();
    let top_level_mark = Mark::new();

    program
      .apply(resolver(unresolved_mark, top_level_mark, true))
      .apply(strip(unresolved_mark, top_level_mark))
      .apply(fixer(None))
  });

  let mut buf = Vec::new();
  {
    let mut emitter = Emitter {
      cfg: Config::default(),
      cm: source_map.clone(),
      comments: None,
      wr: JsWriter::new(source_map, "\n", &mut buf, None),
    };

    emitter
      .emit_program(&program)
      .map_err(|e| format!("TypeScript 输出失败: {}", e))?;
  }

  String::from_utf8(buf).map_err(|e| format!("TypeScript 输出不是合法 utf8: {}", e))
}
