use crate::swc_compiler::SwcCompiler;
use std::path::Path;
use swc_ecma_ast::Program;
use tokio::fs::read_to_string;

#[derive(Debug, Clone)]
pub struct Output {
  pub path: String,
  pub filename: String,
}

// 对齐 Node-Binding 并做一些 Rust 侧的类型适配。（同时也避免循环依赖）
#[derive(Debug, Clone)]
pub struct CompilationOptions {
  pub entry: String,
  pub mode: String,
  pub context: String,
  pub output: Output,
}

#[derive(Debug)]
pub struct Compilation {
  // id: String,
  // TODO: 后面要做多线程的话得加 Arc
  pub options: CompilationOptions,
}

impl Compilation {
  pub fn new(options: CompilationOptions) -> Self {
    println!("\n\nCompilation new: {:?}\n\n", options);
    Self {
      options: options.clone(),
    }
  }

  // 产生 module graph (module + deps)
  pub async fn make(&self) -> Result<(), String> {
    /*
     * 这里 rspack 实际的是把数据结构转为 EntryDependency, 然后通过 ModuleFactory 创建 Module
     * 另外还会开一个 Task Loop 做并行调度
     */
    let entry = self.options.entry.clone();
    let context = Path::new(&self.options.context);
    // PathBuf 类型可以跨平台
    let entry_path = context.join(&entry);

    println!("\n[rust make 阶段] 读取文件: {:?}", entry_path);

    // 使用 tokio::fs::read_to_string，因为是 async 函数
    let source = read_to_string(&entry_path)
      .await
      .map_err(|e| format!("读取文件失败 {:?}: {}", entry_path, e))?;

    println!("文件内容长度: {} 字节", source.len());

    let compiler = SwcCompiler::new();
    let ast = compiler.parse_js(entry_path.clone(), source)?;

    match &ast {
      Program::Module(module) => {
        println!("\n解析到 {} 个语句/声明", module.body.len());

        for item in &module.body {
          use swc_ecma_ast::ModuleItem;
          match item {
            ModuleItem::ModuleDecl(decl) => {
              use swc_ecma_ast::ModuleDecl;
              match decl {
                ModuleDecl::Import(import) => {
                  // 处理 import 语句
                  let source = &import.src.value;
                  println!("  发现 import: {:?}", source);
                  // TODO: 解析依赖路径，创建 Dependency 并添加到 module graph
                }
                ModuleDecl::ExportNamed(_)
                | ModuleDecl::ExportDecl(_)
                | ModuleDecl::ExportAll(_) => {
                  // 处理 export 语句
                  println!("  发现 export 语句");
                }
                _ => {}
              }
            }
            ModuleItem::Stmt(_) => {
              // 普通语句
            }
          }
        }
      }
      Program::Script(_) => {
        return Err("不支持 Script 模式，只支持 Module 模式".to_string());
      }
    }

    // TODO: 创建 Module 并添加到 module graph
    // TODO: 递归处理依赖的模块

    Ok(())
  }

  // make 完成后的检查阶段，但是这里先不实现了
  pub fn finish(&self) {
    todo!()
  }

  // module graph -> chunk graph
  pub fn seal(&self) {
    todo!()
  }
}
