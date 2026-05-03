use swc_common::{sync::Lrc, FileName, SourceMap, DUMMY_SP};
use swc_ecma_ast::{EsVersion, ModuleDecl, ModuleItem, Program};
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

#[derive(Debug, Clone)]
pub struct RawImportRecord {
  pub local: String,
  pub request: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedImportRecord {
  pub local: String,
  pub request: String,
  pub module_id: String,
}

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

  // 源码 -> ast
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

  pub fn collect_imports(&self, program: &Program) -> Result<Vec<RawImportRecord>, String> {
    let mut imports = Vec::new();

    let Program::Module(module) = program else {
      return Err("不支持 Script 模式".into());
    };

    for item in &module.body {
      if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = item {
        let request = import_decl
          .src
          .value
          .as_str()
          .ok_or_else(|| "import 路径不是合法 utf8".to_string())?
          .to_string();

        for specifier in &import_decl.specifiers {
          if let swc_ecma_ast::ImportSpecifier::Default(default_specifier) = specifier {
            imports.push(RawImportRecord {
              local: default_specifier.local.sym.to_string(),
              request: request.clone(),
            });
          }
        }
      }
    }

    Ok(imports)
  }

  // 处理 ast 这段也让 ai 处理了，实在是看力竭了

  pub fn transform_module_ast(
    &self,
    program: Program,
    imports: &[ResolvedImportRecord],
  ) -> Result<Program, String> {
    use swc_ecma_ast::*;

    let Program::Module(module) = program else {
      return Err("不支持 Script 模式".into());
    };

    let mut body = Vec::new();

    for item in module.body {
      match item {
        ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) => {
          let request = import_decl
            .src
            .value
            .as_str()
            .ok_or_else(|| "import 路径不是合法 utf8".to_string())?
            .to_string();

          for specifier in import_decl.specifiers {
            if let ImportSpecifier::Default(default_specifier) = specifier {
              let import_record = imports
                .iter()
                .find(|import| {
                  import.request == request && default_specifier.local.sym == import.local
                })
                .ok_or_else(|| format!("找不到 import 记录: {}", request))?;

              let import_call = Expr::Call(CallExpr {
                span: DUMMY_SP,
                ctxt: Default::default(),
                callee: Callee::Expr(Box::new(Expr::Ident(Ident::new_no_ctxt(
                  "__feopack_import__".into(),
                  DUMMY_SP,
                )))),
                args: vec![ExprOrSpread {
                  spread: None,
                  expr: Box::new(Expr::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: import_record.module_id.clone().into(),
                    raw: None,
                  }))),
                }],
                type_args: None,
              });

              let default_member = Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(import_call),
                prop: MemberProp::Ident(IdentName::from("default")),
              });

              body.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                span: DUMMY_SP,
                ctxt: Default::default(),
                kind: VarDeclKind::Const,
                declare: false,
                decls: vec![VarDeclarator {
                  span: DUMMY_SP,
                  name: Pat::Ident(default_specifier.local.into()),
                  init: Some(Box::new(default_member)),
                  definite: false,
                }],
              })))));
            }
          }
        }

        ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(export_decl)) => {
          match export_decl.decl {
            DefaultDecl::Fn(fn_expr) => {
              let ident = fn_expr
                .ident
                .ok_or_else(|| "暂不支持匿名 export default function".to_string())?;

              body.push(ModuleItem::Stmt(Stmt::Decl(Decl::Fn(FnDecl {
                ident: ident.clone(),
                declare: false,
                function: fn_expr.function,
              }))));

              body.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
                span: DUMMY_SP,
                expr: Box::new(Expr::Call(CallExpr {
                  span: DUMMY_SP,
                  ctxt: Default::default(),
                  callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
                    span: DUMMY_SP,
                    obj: Box::new(Expr::Ident(Ident::new_no_ctxt(
                      "__feopack_import__".into(),
                      DUMMY_SP,
                    ))),
                    prop: MemberProp::Ident(IdentName::from("d")),
                  }))),
                  args: vec![
                    ExprOrSpread {
                      spread: None,
                      expr: Box::new(Expr::Member(MemberExpr {
                        span: DUMMY_SP,
                        obj: Box::new(Expr::Ident(Ident::new_no_ctxt(
                          "__feopack_module__".into(),
                          DUMMY_SP,
                        ))),
                        prop: MemberProp::Ident(IdentName::from("exports")),
                      })),
                    },
                    ExprOrSpread {
                      spread: None,
                      expr: Box::new(Expr::Object(ObjectLit {
                        span: DUMMY_SP,
                        props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                          key: PropName::Ident(IdentName::from("default")),
                          value: Box::new(Expr::Arrow(ArrowExpr {
                            span: DUMMY_SP,
                            ctxt: Default::default(),
                            params: vec![],
                            body: Box::new(BlockStmtOrExpr::Expr(Box::new(Expr::Ident(ident)))),
                            is_async: false,
                            is_generator: false,
                            type_params: None,
                            return_type: None,
                          })),
                        })))],
                      })),
                    },
                  ],
                  type_args: None,
                })),
              })));
            }
            _ => return Err("暂只支持 export default function".into()),
          }
        }

        other => body.push(other),
      }
    }

    Ok(Program::Module(Module {
      span: module.span,
      body,
      shebang: module.shebang,
    }))
  }

  // 这个并非emit阶段用的，而是seal阶段消费module，生成代码用的
  // ast -> js（chunk 内的代码，我理解的是属于chunk内代码到 esm 风格可运行代码的过渡产物）
  // 这里 js -> ast -> js, 中间最大的变化就是，把 js 里面的 import 等语句劫持修改了
  //? 但是怎么想都是一个极其低效的过程，能不能有一种办法，直接一步到位，或者尽可能减小开销呢？
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
