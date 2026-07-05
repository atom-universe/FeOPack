# feopack loader-runner

本目录是 **Rspack loader-runner 教学子集**，不是把 Rspack 整份 1100+ 行搬过来。

## 谱系

```text
webpack/loader-runner
        ↓（Rspack fork + JsLoaderContext 桥接）
packages/rspack/src/loader-runner/
        ↓（feopack mini：保留核心 pitch/normal + 快照接口）
packages/feopack/src/loader-runner/
```

Rspack 完整版还依赖 `@rspack/binding`、`Compilation`、`service.ts`（worker）、`importModule` 等。  
feopack 只保留 **对 mini bundler 够用** 的部分：

| 保留 | 暂不实现 |
|------|----------|
| `runLoaders` / pitch → read → normal | parallel loader worker |
| `JsLoaderContext`（Rust → Node 快照） | 完整 `importModule` / `emitFile` |
| `LoaderContext`（loader 函数里的 `this`） | tinypool / service 全量 RPC |
| `dispatcher`（napi 统一入口） | pitch 混链 yield（待 Rust 接） |

## 与 Rspack 对齐的入口

```ts
// Rspack: runLoaders(compiler, context: JsLoaderContext)
// feopack: runJsLoaders(context: JsLoaderContext)
```

`JsLoaderContext` 字段对齐 Rspack binding 的语义（loaderState、loaders、resource、source…），便于日后 napi 对接。

## 测试

```bash
cd packages/feopack && pnpm test
```

## 参考

- [Rspack loader-runner](https://github.com/web-infra-dev/rspack/tree/main/packages/rspack/src/loader-runner)
- [Rspack loader 架构](https://rspack.rs/contribute/architecture/rspack-loader)
- [webpack/loader-runner](https://github.com/webpack/loader-runner)（原始 MIT 代码）
