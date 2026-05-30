import { FeopackOptions } from '.'
import * as binding from '@feopack/binding'
import { getRawOptions } from './config/adapter'
import { Compilation } from './Complication'
export class Compiler {
  #inner?: binding.Rspack // Rust 那一侧对应的 Compiler
  // 打包的根路径
  options: FeopackOptions
  context: string
  compilation?: Compilation

  constructor(options: FeopackOptions) {
    this.context = options.context
    this.options = options
  }

  #getInner(): binding.Rspack {
    if (this.#inner) {
      return this.#inner
    }
    const rawOptions = getRawOptions(this.options)
    // 拿到 rust 那一侧的 compiler 实例
    const instanceBinding = require('@feopack/binding')

    this.#inner = new instanceBinding.Rspack(
      rawOptions,
      // ThreadsafeWritableNodeFS.__to_binding(this.outputFileSystem!),
      // ResolverFactory.__to_binding(this.resolverFactory),
    )

    return this.#inner!
  }

  async #build() {
    // rust, 启动！
    const inner = this.#getInner()
    await inner.build()
    // TODO: 目前没有插件的部分，所以暂时用不到 nodejs 的 compilation
    // this.compilation = new Compilation(this, inner)
  }

  async compile() {
    await this.#build()
  }

  async run() {
    await this.compile()
  }
}
