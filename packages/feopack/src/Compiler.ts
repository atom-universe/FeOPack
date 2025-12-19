import { FeopackOptions } from '.'
import * as binding from '@feopack/binding'
import { getRawOptions } from './config/adapter'
import { Compilation } from './Complication'
export class Compiler {
  #inner?: binding.Rspack // Rust 那一侧对应的 Compiler
  // 打包的根路径
  options: FeopackOptions
  context: string

  constructor(options: FeopackOptions) {
    this.context = options.context
    this.options = options
  }

  #getInner(): binding.Rspack {
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

    return this.#inner!
  }

  #build(): Compilation {
    const inner = this.#getInner()
    // 让 rust 来执行
    inner.build('test')
    // TODO:暂时创建一个空的 compilation 对象，等 Rust 侧返回 Compilation 后再修改
    return new Compilation(this, { hash: null })
  }

  compile(): Compilation {
    return this.#build()
  }

  run(): Compilation {
    return this.compile()
  }
}
