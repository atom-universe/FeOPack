<div align="center">

# FeOPack

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Node](https://img.shields.io/badge/Node-%3E%3D16-green.svg)](https://nodejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0+-blue.svg)](https://www.typescriptlang.org/)

</div>



## 📖 关于

✨ 本项目为学习 Rspack 的核心功能实现原理以及项目工程架构而编写。

### 为什么叫 FeO？

- **Rust** 意为"铁锈"。
- **FeO** 也是铁锈，但是锈化程度不高，这就有了一层 mini rspack 的含义。


## 🚀 快速开始

### 安装依赖

```bash
pnpmi 
```

### 构建项目

```bash
# 推荐方式（使用 @napi-rs/cli）
npm run build

# 或手动构建
npm run build:napi && npm run build:ts
```

### 使用

```typescript
import { version, compile } from 'feopack';

console.log(version());
console.log(compile('src/index.ts'));
```

## 🛠️ 开发

### 开发模式

```bash
# 监听模式，自动重新构建
npm run dev
```

### 手动构建（不使用 @napi-rs/cli）

```bash
# 构建原生模块
cargo build -p feopack-node

# 构建 TypeScript
npm run build:ts
```

## 📝 说明

这是一个**学习性质**的项目，请勿用于实际生产（相信也没人会这么做）。


## 参考资料

感谢所有相关的公开资料的创作者们的努力！

> 关于这部分，我整理成了文档: https://ai.feishu.cn/wiki/U4imwSfmFimoeGk1qQrck2e5nog?from=from_copylink


## 📄 License

MIT
