use super::{CodegenModule, Compilation, GeneratedAsset, ResolvedPath};
use crate::swc_compiler::{RawImportRecord, ResolvedImportRecord, SwcCompiler};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use swc_ecma_ast::Program;

impl Compilation {
  // 代码生成：为每个模块生成代码并创建 bundle
  pub(crate) async fn code_generation(&mut self) -> Result<(), String> {
    let context = Path::new(&self.options.context);
    let entry_path = context.join(&self.options.entry);
    let entry_module_id = Self::create_module_id(&entry_path)?;

    for chunk in &self.chunk_graph.chunks {
      let compiler = SwcCompiler::new();
      let mut codegen_modules: Vec<CodegenModule> = Vec::new();
      // 外部依赖不进入本地 module graph，运行时交给 Node require 处理。
      let mut external_module_ids = HashSet::new();

      for module_id in &chunk.module_ids {
        let source = self
          .module_sources
          .get(module_id)
          .ok_or_else(|| format!("找不到模块源码: {}", module_id))?
          .clone();
        println!("模块源代码: {:?}", source);

        let (module, imports) = self.generate_module_code(&compiler, module_id, source)?;
        for import in imports {
          if import.external {
            external_module_ids.insert(import.module_id);
          }
        }
        codegen_modules.push(module);
      }

      for module_id in external_module_ids {
        codegen_modules.push(Self::generate_external_module(&module_id));
      }

      let source = Self::render_chunk(&entry_module_id, &codegen_modules);

      self.assets.push(GeneratedAsset {
        filename: self.options.output.filename.clone(),
        source,
      });
    }
    Ok(())
  }

  fn generate_module_code(
    &self,
    compiler: &SwcCompiler,
    module_id: &str,
    source: String,
  ) -> Result<(CodegenModule, Vec<ResolvedImportRecord>), String> {
    // 这里并不是重新 build module，而是消费 make 阶段缓存下来的 transformed source。
    // 目前为了复用 SWC transform，仍然会 source -> ast -> source 走一遍；
    // 后续如果把 AST/BuildResult 存进 Module，就可以继续减少重复 parse。
    let ast: Program = compiler.parse_js(PathBuf::from(module_id), source)?;
    let raw_imports = compiler.collect_imports(&ast)?;
    let imports = self.resolve_imports(module_id, raw_imports)?;
    let transformed_ast = compiler.transform_module_ast(ast, &imports)?;
    let generated_source = compiler.emit_module(&transformed_ast)?;

    Ok((
      CodegenModule {
        id: module_id.to_string(),
        source: generated_source,
      },
      imports,
    ))
  }

  fn generate_external_module(module_id: &str) -> CodegenModule {
    CodegenModule {
      id: module_id.to_string(),
      source: format!(
        r#"const __feopack_external__ = require("{}");
Object.assign(__feopack_exports__, __feopack_external__);
__feopack_import__.d(__feopack_exports__, {{
    default: () => __feopack_external__
}});"#,
        module_id
      ),
    }
  }

  // 之前是 cjs 风格，现在换到 esm 风格
  // 谢谢 ai 帮忙处理（写了几次都有问题）
  // !TODO: 可以包一层 IIFE，防止这里 feopack_modules 和 feopack_cache 污染
  fn render_chunk(entry_module_id: &str, modules: &[CodegenModule]) -> String {
    let mut modules_code = String::new();

    for module in modules {
      let module_source = module
        .source
        .lines()
        .map(|line| format!("    {}", line))
        .collect::<Vec<_>>()
        .join("\n");

      modules_code.push_str(&format!(
        r#"  "{}": (__feopack_module__, __feopack_exports__, __feopack_import__) => {{
{}
  }},
"#,
        module.id, module_source
      ));
    }

    format!(
      r#"
const __feopack_modules__ = {{{}}};
const __feopack_cache__ = {{}};

function __feopack_import__(id) {{
  if (__feopack_cache__[id]) {{
    return __feopack_cache__[id].exports;
  }}

  const __feopack_module__ = {{ exports: {{}} }};
  __feopack_cache__[id] = __feopack_module__;
  __feopack_modules__[id](__feopack_module__, __feopack_module__.exports, __feopack_import__);

  return __feopack_module__.exports;
}}

__feopack_import__.d = (exports, definition) => {{
  for (const key in definition) {{
    if (
      Object.prototype.hasOwnProperty.call(definition, key) &&
      !Object.prototype.hasOwnProperty.call(exports, key)
    ) {{
      Object.defineProperty(exports, key, {{
        enumerable: true,
        get: definition[key],
      }});
    }}
  }}
}};

__feopack_import__("{}");
"#,
      modules_code, entry_module_id
    )
  }

  fn resolve_imports(
    &self,
    module_id: &str,
    raw_imports: Vec<RawImportRecord>,
  ) -> Result<Vec<ResolvedImportRecord>, String> {
    let context = Path::new(&self.options.context);
    let module_path = PathBuf::from(module_id);
    let module_dir = module_path
      .parent()
      .ok_or_else(|| format!("无法获取模块目录: {}", module_id))?;

    // import 是有顺序的，所以这里用 Vec 保留顺序，不用 map。
    let mut imports = Vec::new();

    for raw_import in raw_imports {
      let resolved_path = Self::resolve_path(&raw_import.request, module_dir, context)?;
      let external = matches!(resolved_path, ResolvedPath::External(_));
      let dep_module_id = match resolved_path {
        ResolvedPath::File(resolved_module) => resolved_module.module_id,
        ResolvedPath::External(module_id) => module_id,
      };

      imports.push(ResolvedImportRecord {
        local: raw_import.local,
        imported: raw_import.imported,
        request: raw_import.request,
        module_id: dep_module_id,
        external,
      });
    }

    Ok(imports)
  }
}
