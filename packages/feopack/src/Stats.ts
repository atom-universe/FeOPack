import { Compilation } from './Complication'

export class Stats {
  compilation: Compilation
  // #innerMap: WeakMap<Compilation, any>

  constructor(compilation: Compilation) {
    this.compilation = compilation
    // this.#innerMap = new WeakMap([[this.compilation, this.#inner]])
  }

  // use correct JsStats for child compilation
  // #getInnerByCompilation(compilation: Compilation): any {
  //   if (this.#innerMap.has(compilation)) {
  //     return this.#innerMap.get(compilation)!
  //   }
  //   const inner = compilation.__internal_getInner().getStats()
  //   this.#innerMap.set(compilation, inner)
  //   return inner
  // }

  get hash() {
    return null
  }

  // get startTime() {
  //   return this.compilation.startTime
  // }

  // get endTime() {
  //   return this.compilation.endTime
  // }

  hasErrors() {
    return false
  }

  hasWarnings() {
    return false
  }
}
