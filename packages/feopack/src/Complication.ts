import type { Compiler } from './Compiler'
import { Stats } from './Stats'
import * as binding from '@feopack/binding'
export class Compilation {
  // 再 rust 那一侧的 compilation 实例
  name?: string
  #inner: binding.Rspack
  compiler: Compiler
  // options: FeopackOptions
  // outputOptions: OutputNormalized
  // startTime?: number
  // endTime?: number

  constructor(compiler: Compiler, inner: binding.Rspack) {
    this.#inner = inner
    this.compiler = compiler
    // this.options = compiler.options
    // this.outputOptions = compiler.options.output
  }

  // get hash(): string | null {
  //   return this.#inner.hash
  // }

  getStats(): Stats {
    return new Stats(this)
  }

  get fileDependencies(): ReadonlySet<string> {
    return new Set(this.#inner.getFileDependencies())
  }

  __internal_getInner(): binding.Rspack {
    return this.#inner
  }
}
