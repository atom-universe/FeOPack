import { Compilation } from './Complication'

export class Stats {
  // rust 那一侧的 stats
  #inner: any
  compilation: Compilation
  #innerMap: WeakMap<Compilation, any>

  constructor(compilation: Compilation) {
    this.#inner = compilation.__internal_getInner().getStats()
    this.compilation = compilation
    this.#innerMap = new WeakMap([[this.compilation, this.#inner]])
  }

  // use correct JsStats for child compilation
  #getInnerByCompilation(compilation: Compilation): any {
    if (this.#innerMap.has(compilation)) {
      return this.#innerMap.get(compilation)!
    }
    const inner = compilation.__internal_getInner().getStats()
    this.#innerMap.set(compilation, inner)
    return inner
  }

  get hash() {
    return this.compilation.hash
  }

  // get startTime() {
  //   return this.compilation.startTime
  // }

  // get endTime() {
  //   return this.compilation.endTime
  // }

  hasErrors() {
    return this.#inner.hasErrors()
  }

  hasWarnings() {
    return this.#inner.hasWarnings()
  }
}
