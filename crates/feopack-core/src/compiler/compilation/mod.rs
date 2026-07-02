mod code_generation;
mod make;
mod resolve;
mod seal;

use crate::loader::text_loader::text_loader;
use crate::loader::meow_loader_v1::meow_loader_v1;
use crate::loader::meow_loader_v2::{
  meow_extract_script, meow_extract_style, meow_extract_template, meow_loader_v2_main,
  meow_scope_style, meow_wrap_script_export, meow_wrap_style_export, meow_wrap_template_export,
};
use crate::loader::meow_loader_v3::{meow_loader_v3_main, meow_v3_pitch, meow_v3_pitcher_normal};
use crate::loader::typescript_loader::typescript_loader;
use crate::loader::{Loader, LoaderRegistry, LoaderRule};
use crate::loader::inline_request::InlineRequest;
use crate::module_graph::ModuleGraph;
use std::collections::HashMap;
use std::path::PathBuf;

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

#[derive(Debug, Clone)]
pub struct Chunk {
  pub id: String,
  pub module_ids: Vec<String>,
}

#[derive(Debug, Default)]
pub struct ChunkGraph {
  pub chunks: Vec<Chunk>,
}

#[derive(Debug, Clone)]
pub struct GeneratedAsset {
  pub filename: String,
  pub source: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CodegenModule {
  // code_generation 阶段消费的临时结构：
  // 到这里已经不是“源码文件”，而是即将塞进 chunk runtime 的模块函数体。
  pub(crate) id: String,
  pub(crate) source: String,
}

#[derive(Debug, Clone)]
pub(crate) enum ResolvedPath {
  // File(PathBuf),
  File(ResolvedModule),
  External(String),
}

// PathBuf 逐渐不够用了，改造成 RsolvedModule
#[derive(Debug, Clone)]
pub(crate) struct ResolvedModule {
  // 也就是 /abs/xxx?yyy=123 这样的内容
  pub module_id: String,
  // /abs/xxx
  pub resource_path: PathBuf,
  // ?yyy=123
  pub resource_query: String,
  pub inline: InlineRequest,
}


#[derive(Debug)]
pub struct Compilation {
  pub options: CompilationOptions,
  pub module_graph: ModuleGraph,
  pub chunk_graph: ChunkGraph,
  pub assets: Vec<GeneratedAsset>,
  // make 阶段的 build result 先简单放这里。
  // 真实 bundler 通常会把这类信息挂在 Module/BuildResult 上；这里先用 map 保持 MVP 清晰。
  pub(crate) module_sources: HashMap<String, String>,
  pub(crate) loader_registry: LoaderRegistry,
}

impl Compilation {
  pub fn new(options: CompilationOptions) -> Self {
    println!("\n\nCompilation new: {:?}\n\n", options);
    let mut loader_registry = LoaderRegistry::new();

    loader_registry.register_loader("text-loader".to_string(), Loader::normal_only(text_loader));
    loader_registry.register_loader(
      "meow-loader-v1".to_string(),
      Loader::normal_only(meow_loader_v1),
    );
    loader_registry.register_loader(
      "meow-loader-v2-main".to_string(),
      Loader::normal_only(meow_loader_v2_main),
    );
    loader_registry.register_loader(
      "meow-extract-template".to_string(),
      Loader::normal_only(meow_extract_template),
    );
    loader_registry.register_loader(
      "meow-extract-script".to_string(),
      Loader::normal_only(meow_extract_script),
    );
    loader_registry.register_loader(
      "meow-wrap-template-export".to_string(),
      Loader::normal_only(meow_wrap_template_export),
    );
    loader_registry.register_loader(
      "meow-wrap-script-export".to_string(),
      Loader::normal_only(meow_wrap_script_export),
    );
    loader_registry.register_loader(
      "meow-extract-style".to_string(),
      Loader::normal_only(meow_extract_style),
    );
    loader_registry.register_loader(
      "meow-scope-style".to_string(),
      Loader::normal_only(meow_scope_style),
    );
    loader_registry.register_loader(
      "meow-wrap-style-export".to_string(),
      Loader::normal_only(meow_wrap_style_export),
    );
    loader_registry.register_loader(
      "typescript-loader".to_string(),
      Loader::normal_only(typescript_loader),
    );

    // meow-v3：pitch + resourceQuery rule（子 block 不再写 inline loader 链）
    loader_registry.register_loader(
      "meow-v3-pitcher".to_string(),
      Loader::with_pitch(meow_v3_pitch, meow_v3_pitcher_normal),
    );
    loader_registry.register_loader(
      "meow-loader-v3-main".to_string(),
      Loader::normal_only(meow_loader_v3_main),
    );
    loader_registry.register_loader(
      "meow-v3-extract-template".to_string(),
      Loader::normal_only(meow_extract_template),
    );
    loader_registry.register_loader(
      "meow-v3-extract-script".to_string(),
      Loader::normal_only(meow_extract_script),
    );
    loader_registry.register_loader(
      "meow-v3-extract-style".to_string(),
      Loader::normal_only(meow_extract_style),
    );
    loader_registry.register_loader(
      "meow-v3-scope-style".to_string(),
      Loader::normal_only(meow_scope_style),
    );
    loader_registry.register_loader(
      "meow-v3-wrap-template-export".to_string(),
      Loader::normal_only(meow_wrap_template_export),
    );
    loader_registry.register_loader(
      "meow-v3-wrap-script-export".to_string(),
      Loader::normal_only(meow_wrap_script_export),
    );
    loader_registry.register_loader(
      "meow-v3-wrap-style-export".to_string(),
      Loader::normal_only(meow_wrap_style_export),
    );

    loader_registry.add_rule(LoaderRule {
      test: ".txt".to_string(),
      resource_query: String::new(),
      used_loaders: vec!["text-loader".to_string()],
    });
    loader_registry.add_rule(LoaderRule {
      test: ".meow-v1".to_string(),
      resource_query: String::new(),
      used_loaders: vec!["meow-loader-v1".to_string()],
    });

    // inline loader 之前：每个 virtual request 都要在这里手写一条 resourceQuery rule。
    // 缺点：lang=ts / lang=js、scoped / 非 scoped 各要一条，和 meow-loader 强耦合，typescript-loader 也得重复写进 rule。
    //
    // 现在：子 block 的 loader 链写在 meow-loader-v2-main 生成的 import 字符串里，例如：
    //   import __script__ from '-!meow-wrap-script-export!typescript-loader!meow-extract-script!./index.meow-v2?type=script&lang=ts';
    // 好处：
    //   1. 这里只需注册 .meow-v2 主请求一条 rule
    //   2. typescript-loader 只作为 inline 复用，不必为每种 block 重复注册
    //   3. 和 webpack 的 `-!loader!resource` 机制对齐，后续 pitch 可以在此基础上继续演进
    //
    // loader_registry.add_rule(LoaderRule {
    //   test: ".meow-v2".to_string(),
    //   resource_query: "?type=script&lang=ts".to_string(),
    //   used_loaders: vec![
    //     "meow-wrap-script-export".to_string(),
    //     "typescript-loader".to_string(),
    //     "meow-extract-script".to_string(),
    //   ],
    // });
    // loader_registry.add_rule(LoaderRule {
    //   test: ".meow-v2".to_string(),
    //   resource_query: "?type=script&lang=js".to_string(),
    //   used_loaders: vec![
    //     "meow-wrap-script-export".to_string(),
    //     "meow-extract-script".to_string(),
    //   ],
    // });
    // loader_registry.add_rule(LoaderRule {
    //   test: ".meow-v2".to_string(),
    //   resource_query: "?type=style&scoped".to_string(),
    //   used_loaders: vec![
    //     "meow-wrap-style-export".to_string(),
    //     "meow-scope-style".to_string(),
    //     "meow-extract-style".to_string(),
    //   ],
    // });
    // loader_registry.add_rule(LoaderRule {
    //   test: ".meow-v2".to_string(),
    //   resource_query: "?type=style".to_string(),
    //   used_loaders: vec![
    //     "meow-wrap-style-export".to_string(),
    //     "meow-extract-style".to_string(),
    //   ],
    // });
    // loader_registry.add_rule(LoaderRule {
    //   test: ".meow-v2".to_string(),
    //   resource_query: "?type=template".to_string(),
    //   used_loaders: vec![
    //     "meow-wrap-template-export".to_string(),
    //     "meow-extract-template".to_string(),
    //   ],
    // });
    loader_registry.add_rule(LoaderRule {
      test: ".meow-v2".to_string(),
      resource_query: String::new(),
      used_loaders: vec!["meow-loader-v2-main".to_string()],
    });

    // meow-v3：pitch 认 query + rule 配 loader 链（对比 v2 的 inline import 方案）
    loader_registry.add_rule(LoaderRule {
      test: ".meow-v3".to_string(),
      resource_query: String::new(),
      used_loaders: vec![
        "meow-v3-pitcher".to_string(),
        "meow-loader-v3-main".to_string(),
      ],
    });
    loader_registry.add_rule(LoaderRule {
      test: ".meow-v3".to_string(),
      resource_query: "?type=script&lang=ts".to_string(),
      used_loaders: vec![
        "meow-v3-pitcher".to_string(),
        "meow-v3-wrap-script-export".to_string(),
        "typescript-loader".to_string(),
        "meow-v3-extract-script".to_string(),
      ],
    });
    loader_registry.add_rule(LoaderRule {
      test: ".meow-v3".to_string(),
      resource_query: "?type=script&lang=js".to_string(),
      used_loaders: vec![
        "meow-v3-pitcher".to_string(),
        "meow-v3-wrap-script-export".to_string(),
        "meow-v3-extract-script".to_string(),
      ],
    });
    loader_registry.add_rule(LoaderRule {
      test: ".meow-v3".to_string(),
      resource_query: "?type=template".to_string(),
      used_loaders: vec![
        "meow-v3-pitcher".to_string(),
        "meow-v3-wrap-template-export".to_string(),
        "meow-v3-extract-template".to_string(),
      ],
    });
    loader_registry.add_rule(LoaderRule {
      test: ".meow-v3".to_string(),
      resource_query: "?type=style&scoped".to_string(),
      used_loaders: vec![
        "meow-v3-pitcher".to_string(),
        "meow-v3-wrap-style-export".to_string(),
        "meow-v3-scope-style".to_string(),
        "meow-v3-extract-style".to_string(),
      ],
    });
    loader_registry.add_rule(LoaderRule {
      test: ".meow-v3".to_string(),
      resource_query: "?type=style".to_string(),
      used_loaders: vec![
        "meow-v3-pitcher".to_string(),
        "meow-v3-wrap-style-export".to_string(),
        "meow-v3-extract-style".to_string(),
      ],
    });

    loader_registry.add_rule(LoaderRule {
      test: ".ts".to_string(),
      resource_query: String::new(),
      used_loaders: vec!["typescript-loader".to_string()],
    });

    Self {
      options: options.clone(),
      module_graph: ModuleGraph::new(),
      chunk_graph: ChunkGraph::default(),
      assets: Vec::new(),
      module_sources: HashMap::new(),
      loader_registry,
    }
  }

  pub async fn make(&mut self) -> Result<(), String> {
    self.build_module_graph().await
  }

  pub async fn seal(&mut self) -> Result<(), String> {
    // module graph 到这里以后先视为稳定，seal 负责把它组织成 chunk，并继续生成 assets。
    println!(
      "\n[rust seal 阶段] module graph -> chunk graph:\n {:?}\n",
      self.module_graph.partials
    );
    self.create_chunk_graph().await;
    self.code_generation().await?;
    Ok(())
  }
}
