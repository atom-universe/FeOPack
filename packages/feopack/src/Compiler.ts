import { FeopackOptions } from '.'
import * as binding from '@feopack/binding'
import { getRawOptions } from './config/adapter'
import { Compilation } from './Complication'
import {
  createDefaultJsLoaderDispatcher,
  registerJsLoaderDispatcher,
} from './loader-runner/dispatcher'
import { runJsLoaders } from './loader-runner/snapshot'
import { JsLoaderState } from './loader-runner/types'
import { CompilerHooks, createCompilerHooks } from './hooks'

export class Compiler {
  #inner?: binding.Rspack // Rust 那一侧对应的 Compiler
  // 打包的根路径
  options: FeopackOptions
  context: string
  compilation?: Compilation
  hooks: CompilerHooks
  webpack: Record<string, never>
  rspack: Record<string, never>

  constructor(options: FeopackOptions) {
    this.context = options.context
    this.options = options
    this.hooks = createCompilerHooks()
    this.webpack = {}
    this.rspack = {}
    registerJsLoaderDispatcher(createDefaultJsLoaderDispatcher())
    this.#applyPlugins()
  }

  #applyPlugins() {
    for (const plugin of this.options.plugins ?? []) {
      if (typeof plugin === 'function') {
        ;(plugin as (this: Compiler, compiler: Compiler) => void).call(this, this)
        continue
      }
      plugin.apply(this)
    }
  }

  #getInner(): binding.Rspack {
    if (this.#inner) {
      return this.#inner
    }
    const rawOptions = getRawOptions(this.options)
    // 拿到 rust 那一侧的 compiler 实例
    const instanceBinding = require('@feopack/binding')

    // 把 js loader runner 传递给 rust 那一侧
    this.#inner = new instanceBinding.Rspack(
      rawOptions,
      async (ctx: binding.JsLoaderContextInput) => {
        try {
          // 其实这个玩意儿的核心实现就是直接 fork 的 webpack 的源码
          // 这里其实就是之前说的，能够逆向让 rust 调用 js 的操作的体现之一
          const result = await runJsLoaders({
            loaderState:
              ctx.loaderState === 'pitching'
                ? JsLoaderState.Pitching
                : JsLoaderState.Normal,
            loaders: ctx.loaders,
            resource: ctx.resource,
            source: ctx.source,
            projectRoot: ctx.projectRoot,
            skipReadResource: ctx.skipReadResource,
          })
          return {
            source: result.source,
            shortCircuit: result.shortCircuit,
            pitchedLoaderIndex: result.pitchedLoaderIndex,
          }
        } catch (err) {
          const message = err instanceof Error ? err.message : String(err)
          throw new Error(`feopack js loader: ${message}`)
        }
      },
    )

    return this.#inner!
  }

  async #build() {
    const compilationStub = { compiler: this }

    await this.hooks.beforeRun.promise(this)
    await this.hooks.beforeCompile.promise({})
    await this.hooks.make.promise(compilationStub)
    await this.hooks.compilation.promise(compilationStub)

    // rust, 启动！
    const inner = this.#getInner()
    try {
      await inner.build()
    } catch (err) {
      this.hooks.failed.call(err)
      throw err
    }

    this.compilation = new Compilation(this, inner)
    await this.hooks.afterEmit.promise(this.compilation)
    const stats = this.compilation.getStats()
    await this.hooks.done.promise(stats)
    this.hooks.afterDone.call(stats)
  }

  async compile() {
    await this.#build()
  }

  async run() {
    await this.compile()
  }
}
