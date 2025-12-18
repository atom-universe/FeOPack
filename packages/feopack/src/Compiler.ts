import { FeopackOptions } from '.'
import { getRawOptions } from './config/adapter'
import { Compilation } from './Complication'

export class Compiler {
  #inner: any // Rust 那一侧对应的 Compiler
  // 打包的根路径
  options: FeopackOptions
  context: string

  constructor(options: FeopackOptions) {
    this.context = options.context
    this.options = options
  }

  #getInner(): any {
    if (this.#inner) {
      return this.#inner
    }
    const rawOptions = getRawOptions(this.options)
    const instanceBinding = require('@feopack/binding')

    this.#inner = new instanceBinding.Rspack(
      rawOptions,
      // ThreadsafeWritableNodeFS.__to_binding(this.outputFileSystem!),
      // ResolverFactory.__to_binding(this.resolverFactory),
    )
    return this.#inner
  }

  #build(): Compilation {
    const inner = this.#getInner()
    // 让 rust 来执行
    const rustCompilation = inner.build()
    return new Compilation(this, rustCompilation)
  }

  compile(): Compilation {
    return this.#build()
  }

  run(): Compilation {
    return this.compile()
  }
}
