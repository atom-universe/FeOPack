// Mini Compilation - 最简版
import type { Compiler } from './Compiler'
import { Stats } from './Stats'

export class Compilation {
  // 再 rust 那一侧的 compilation 实例
  name?: string
  #inner: any
  compiler: Compiler
  // options: FeopackOptions
  // outputOptions: OutputNormalized
  // startTime?: number
  // endTime?: number

  constructor(compiler: Compiler, inner: any) {
    this.#inner = inner
    this.compiler = compiler
    // this.options = compiler.options
    // this.outputOptions = compiler.options.output
  }

  get hash(): string | null {
    return this.#inner.hash
  }

  getStats(): Stats {
    return new Stats(this)
  }

  __internal_getInner(): any {
    return this.#inner
  }
}
