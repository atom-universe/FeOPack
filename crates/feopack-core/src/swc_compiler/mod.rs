use std::collections::HashMap;

use swc_common::{sync::Lrc, FileName, SourceMap, DUMMY_SP};
use swc_ecma_ast::{EsVersion, ModuleDecl, ModuleItem, Program};
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_visit::{VisitMut, VisitMutWith};

#[derive(Debug, Clone)]
pub struct RawImportRecord {
  pub local: String,
  pub imported: String,
  pub request: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedImportRecord {
  // import a from 'xx/xxx',
  // local 就是其中的中的 a
  pub local: String,
  // export 的方式，主要是区分是否为 default
  pub imported: String,
  // 其中的 './xx/xxx'
  pub request: String,
  // request 字段对应的完整路径 'User/阿巴巴/xx/xxx'
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
          match specifier {
            swc_ecma_ast::ImportSpecifier::Default(default_specifier) => {
              imports.push(RawImportRecord {
                local: default_specifier.local.sym.to_string(),
                imported: "default".to_string(),
                request: request.clone(),
              });
            }
            swc_ecma_ast::ImportSpecifier::Named(named_specifier) => {
              let imported = named_specifier
                .imported
                .as_ref()
                .map(|imported| imported.atom().to_string())
                .unwrap_or_else(|| named_specifier.local.sym.to_string());

              imports.push(RawImportRecord {
                local: named_specifier.local.sym.to_string(),
                imported,
                request: request.clone(),
              });
            }
            _ => {}
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
    let mut import_bindings = HashMap::new();
    let mut import_index = 0usize;

    for item in module.body {
      match item {
        ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) => {
          let request = import_decl
            .src
            .value
            .as_str()
            .ok_or_else(|| "import 路径不是合法 utf8".to_string())?
            .to_string();

          let import_record = imports
            .iter()
            .find(|import| import.request == request)
            .ok_or_else(|| format!("找不到 import 记录: {}", request))?;
          let namespace = format!("__feopack_import_{}__", import_index);
          import_index += 1;

          body.push(Self::create_import_namespace_decl(
            &namespace,
            &import_record.module_id,
          ));

          for specifier in import_decl.specifiers {
            match specifier {
              ImportSpecifier::Default(default_specifier) => {
                import_bindings.insert(
                  default_specifier.local.sym.to_string(),
                  (namespace.clone(), "default".to_string()),
                );
              }
              ImportSpecifier::Named(named_specifier) => {
                let imported = named_specifier
                  .imported
                  .as_ref()
                  .map(|imported| imported.atom().to_string())
                  .unwrap_or_else(|| named_specifier.local.sym.to_string());

                import_bindings.insert(
                  named_specifier.local.sym.to_string(),
                  (namespace.clone(), imported),
                );
              }
              _ => return Err("暂不支持 namespace import".into()),
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
              body.push(Self::create_define_exports_stmt(vec![(
                "default".to_string(),
                ident.sym.to_string(),
              )]));
            }
            _ => return Err("暂只支持 export default function".into()),
          }
        }

        ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl)) => match export_decl.decl {
          Decl::Var(var_decl) => {
            let exports = Self::collect_var_decl_exports(&var_decl)?;
            body.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(var_decl))));
            body.push(Self::create_define_exports_stmt(exports));
          }
          Decl::Fn(fn_decl) => {
            let export_name = fn_decl.ident.sym.to_string();
            body.push(ModuleItem::Stmt(Stmt::Decl(Decl::Fn(fn_decl))));
            body.push(Self::create_define_exports_stmt(vec![(
              export_name.clone(),
              export_name,
            )]));
          }
          _ => return Err("暂只支持 export var/function".into()),
        },

        ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(named_export)) => {
          if named_export.src.is_some() {
            return Err("暂不支持 re-export from".into());
          }

          let mut exports = Vec::new();
          for specifier in named_export.specifiers {
            match specifier {
              ExportSpecifier::Named(named_specifier) => {
                let local = named_specifier.orig.atom().to_string();
                let exported = named_specifier
                  .exported
                  .as_ref()
                  .map(|exported| exported.atom().to_string())
                  .unwrap_or_else(|| local.clone());
                exports.push((exported, local));
              }
              _ => return Err("暂不支持当前 export specifier".into()),
            }
          }

          if !exports.is_empty() {
            body.push(Self::create_define_exports_stmt(exports));
          }
        }

        other => body.push(other),
      }
    }

    let mut transformed_module = Module {
      span: module.span,
      body,
      shebang: module.shebang,
    };

    transformed_module.visit_mut_with(&mut ImportedBindingRewriter {
      bindings: import_bindings,
    });

    Ok(Program::Module(transformed_module))
  }

  fn create_import_namespace_decl(namespace: &str, module_id: &str) -> ModuleItem {
    use swc_ecma_ast::*;

    ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
      span: DUMMY_SP,
      ctxt: Default::default(),
      kind: VarDeclKind::Const,
      declare: false,
      decls: vec![VarDeclarator {
        span: DUMMY_SP,
        name: Pat::Ident(BindingIdent::from(Ident::new_no_ctxt(
          namespace.into(),
          DUMMY_SP,
        ))),
        init: Some(Box::new(Expr::Call(CallExpr {
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
              value: module_id.into(),
              raw: None,
            }))),
          }],
          type_args: None,
        }))),
        definite: false,
      }],
    }))))
  }

  fn collect_var_decl_exports(
    var_decl: &swc_ecma_ast::VarDecl,
  ) -> Result<Vec<(String, String)>, String> {
    use swc_ecma_ast::*;

    let mut exports = Vec::new();
    for declarator in &var_decl.decls {
      match &declarator.name {
        Pat::Ident(binding_ident) => {
          let name = binding_ident.id.sym.to_string();
          exports.push((name.clone(), name));
        }
        _ => return Err("暂不支持解构 export var".into()),
      }
    }

    Ok(exports)
  }

  fn create_define_exports_stmt(exports: Vec<(String, String)>) -> ModuleItem {
    use swc_ecma_ast::*;

    ModuleItem::Stmt(Stmt::Expr(ExprStmt {
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
              props: exports
                .into_iter()
                .map(|(exported, local)| {
                  PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(IdentName::new(exported.into(), DUMMY_SP)),
                    value: Box::new(Expr::Arrow(ArrowExpr {
                      span: DUMMY_SP,
                      ctxt: Default::default(),
                      params: vec![],
                      body: Box::new(BlockStmtOrExpr::Expr(Box::new(Expr::Ident(
                        Ident::new_no_ctxt(local.into(), DUMMY_SP),
                      )))),
                      is_async: false,
                      is_generator: false,
                      type_params: None,
                      return_type: None,
                    })),
                  })))
                })
                .collect(),
            })),
          },
        ],
        type_args: None,
      })),
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

struct ImportedBindingRewriter {
  bindings: HashMap<String, (String, String)>,
}

impl VisitMut for ImportedBindingRewriter {
  fn visit_mut_expr(&mut self, expr: &mut swc_ecma_ast::Expr) {
    use swc_ecma_ast::*;

    if let Expr::Ident(ident) = expr {
      if let Some((namespace, imported)) = self.bindings.get(ident.sym.as_ref()) {
        *expr = Expr::Member(MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(Expr::Ident(Ident::new_no_ctxt(
            namespace.clone().into(),
            DUMMY_SP,
          ))),
          prop: MemberProp::Ident(IdentName::new(imported.clone().into(), DUMMY_SP)),
        });
        return;
      }
    }

    expr.visit_mut_children_with(self);
  }
}
