use crate::loader::LoaderContext;


pub fn text_loader(context: LoaderContext) -> Result<String, String> {
  Ok(format!(
    "const __feopack_text__ = {:?};\nexport {{ __feopack_text__ as default }};",
    context.source
  ))
}


// 之前我是这样写的：
// pub fn text_loader(context: LoaderContext) -> Result<String, String> {
//   Ok(format!("export default {:?}", context.source))
// }
// 实际执行的时候代码会被处理成：
// export default "Hello FeOPack Loader";

// 本身没啥问题
// 但是目前的 swc 竟然，竟然不许！
// SwcCompiler::transform_module_ast 不能直接转换这样的结构（这没被识别为一个module？）
// 它目前只支持大概长这样的结构：
// export default xxx
// export const xxx = xxx
// export function xxx() {}
// 要改 ast 的话太麻烦了，还是我们 loader 这边先妥协一下吧