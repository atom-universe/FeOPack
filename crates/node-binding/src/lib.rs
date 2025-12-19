#![deny(clippy::all)]

use feopack_binding::options::RawOptions;
use napi::Result;
use napi_derive::napi;

#[napi]
pub struct Rspack {}

#[napi]
impl Rspack {
  #[napi(constructor)]
  pub fn new(options: RawOptions) -> Result<Self> {
    // TODO: 从 options 中解析配置

    // let rspack = feopack_core::Compiler::new(
    //   options,
    //   // plugins,
    //   // AsyncNodeWritableFileSystem::new(output_filesystem)
    //   //   .map_err(|e| Error::from_reason(format!("Failed to create writable filesystem: {e}",)))?,
    //   // Some(resolver_factory),
    //   // Some(loader_resolver_factory),
    // );

    // Ok(Self {
    //   compiler: Box::pin(Compiler::from(rspack)),
    //   // state: CompilerState::init(),
    //   // js_plugin,
    // })
    Ok(Self {})
  }

  #[napi]
  pub fn build(&self, test_string: String) -> Result<()> {
    println!("test_string: {}", test_string);
    Ok(())
  }
}

// build() 方法，返回 Compilation
//   #[napi]
//   pub fn build(&mut self) -> napi::Result<Compilation> {
//     // TODO: 执行编译逻辑
//     // 返回 Compilation 实例
//     Ok(Compilation { hash: None })
//   }
// }

// Compilation 类，对应 JS 侧的 Compilation
// #[napi]
// pub struct Compilation {
//   hash: Option<String>,
// }

// #[napi]
// impl Compilation {
//   #[napi(getter)]
//   pub fn hash(&self) -> Option<String> {
//     self.hash.clone()
//   }

//   #[napi]
//   pub fn get_stats(&mut self) -> napi::Result<Stats> {
//     // TODO: 从 Rust 侧获取实际的 stats 数据
//     Ok(Stats {
//       has_errors: false,
//       has_warnings: false,
//     })
//   }
// }

// // Stats 类
// #[napi]
// pub struct Stats {
//   has_errors: bool,
//   has_warnings: bool,
// }

// #[napi]
// impl Stats {
//   #[napi]
//   pub fn has_errors(&self) -> bool {
//     self.has_errors
//   }

//   #[napi]
//   pub fn has_warnings(&self) -> bool {
//     self.has_warnings
//   }
// }
