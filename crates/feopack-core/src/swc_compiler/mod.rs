use swc_common::{sync::Lrc, FileName, SourceMap};
use swc_ecma_ast::{EsVersion, Program};
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

pub struct SwcCompiler {
  // SourceMap 是 swc 的源码管理器，用于管理源码文件和位置信息
  // 比如 AST 里的某个节点来自源文件第几行、第几列
  // Lrc 则是 SWC 对 Arc 的封装，让多处可以共享一个 source_map
  // source_map: Arc<SourceMap>,
  source_map: Lrc<SourceMap>,
}

impl SwcCompiler {
  pub fn new() -> Self {
    Self {
      source_map: Lrc::new(SourceMap::default()),
    }
  }

  pub fn parse_js(&self, file_path: std::path::PathBuf, source: String) -> Result<Program, String> {
    // 1. 创建 SourceFile
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

  // 这个并非emit阶段用的，而是seal阶段消费module，生成代码用的
  pub fn emit_module(&self, program: &Program) -> Result<String, String> {
    // 哎，似乎 swc 各种生态版本兼容对齐也是一个难题诶，to_code 用不了——死去的回忆开始攻击我
    // Ok(to_code(program))
    // 这里开一个 vec，然后让 swc 的 ast 迭代器遍历 ast 然后把有用的东西丢进来
    let mut buf = Vec::new();
    {
      // Emitter 是 SWC codegen 的核心对象。
      // 负责遍历 AST，然后把对应的 JS 代码写入 writer。
      let mut emitter = Emitter {
        // codegen 配置。
        // 这里先用默认配置即可。
        cfg: Config::default(),

        // 传入 SourceMap。
        // clone() 这里不是复制整个 SourceMap，
        // 而是复制一份 Lrc 指针，让 emitter 也共享同一个 SourceMap。
        cm: self.source_map.clone(),

        // comments 用来控制注释输出。
        // 这里先不处理注释，所以传 None。
        comments: None,

        // JsWriter 是真正负责“写 JS 文本”的 writer。
        // 1. self.source_map.clone(): SourceMap
        // 3. &mut buf: 把生成结果写进 buf
        // 4. None: 不生成 sourcemap
        wr: JsWriter::new(self.source_map.clone(), "\n", &mut buf, None),
      };

      // emit_program 会把 Program AST 打印成 JS。
      // 这里的 ? 表示：
      // 如果 emit_program 返回 Err，就提前 return Err(...)
      // 如果返回 Ok，就继续往下执行。
      //
      // map_err 是把 SWC 的错误类型转换成我们自己的 String 错误。
      emitter
        .emit_program(program)
        .map_err(|e| format!("emit module 失败: {}", e))?;
    }

    // 收集到的内容直接转换为 string
    String::from_utf8(buf).map_err(|e| format!("emit module 结果不是合法 utf8: {}", e))
  }
}
