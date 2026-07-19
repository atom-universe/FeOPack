// 这里参考的是 webpack/tapable 的 hook 模型，Rspack 的 JS Compiler 也是沿用这一套思路。
// Feopack 目前只保留学习 JS plugin 所需的最小子集：注册 tap，然后按顺序调用。
// 不处理 stage、interceptor、HookMap 等完整能力，后面真的遇到 case 再补。

type Callback = (err?: Error | null) => void

type TapKind = 'sync' | 'async' | 'promise'

type Tap = {
  name: string
  kind: TapKind
  fn: (...args: any[]) => unknown
}

function normalizeTapName(options: string | { name?: string }): string {
  if (typeof options === 'string') {
    return options
  }
  return options.name ?? 'anonymous'
}

export class MiniSeriesHook {
  #taps: Tap[] = []

  tap(options: string | { name?: string }, fn: (...args: any[]) => unknown) {
    this.#taps.push({ name: normalizeTapName(options), kind: 'sync', fn })
  }

  tapAsync(
    options: string | { name?: string },
    fn: (...args: [...any[], Callback]) => unknown,
  ) {
    this.#taps.push({ name: normalizeTapName(options), kind: 'async', fn })
  }

  tapPromise(
    options: string | { name?: string },
    fn: (...args: any[]) => Promise<unknown>,
  ) {
    this.#taps.push({ name: normalizeTapName(options), kind: 'promise', fn })
  }

  isUsed(): boolean {
    return this.#taps.length > 0
  }

  call(...args: unknown[]) {
    for (const tap of this.#taps) {
      tap.fn(...args)
    }
  }

  async promise(...args: unknown[]) {
    for (const tap of this.#taps) {
      await this.#runTap(tap, args)
    }
  }

  callAsync(...args: [...unknown[], Callback]) {
    const callback = args[args.length - 1] as Callback
    const hookArgs = args.slice(0, -1)

    this.promise(...hookArgs).then(
      () => callback(),
      (err) => callback(err instanceof Error ? err : new Error(String(err))),
    )
  }

  async #runTap(tap: Tap, args: unknown[]) {
    if (tap.kind === 'async') {
      await new Promise<void>((resolve, reject) => {
        tap.fn(...args, (err?: Error | null) => {
          if (err) {
            reject(err)
            return
          }
          resolve()
        })
      })
      return
    }

    await tap.fn(...args)
  }
}

export function createCompilerHooks() {
  return {
    thisCompilation: new MiniSeriesHook(),
    beforeRun: new MiniSeriesHook(),
    run: new MiniSeriesHook(),
    beforeCompile: new MiniSeriesHook(),
    make: new MiniSeriesHook(),
    compilation: new MiniSeriesHook(),
    afterCompile: new MiniSeriesHook(),
    emit: new MiniSeriesHook(),
    afterEmit: new MiniSeriesHook(),
    assetEmitted: new MiniSeriesHook(),
    done: new MiniSeriesHook(),
    afterDone: new MiniSeriesHook(),
    failed: new MiniSeriesHook(),
    watchRun: new MiniSeriesHook(),
    watchClose: new MiniSeriesHook(),
    compile: new MiniSeriesHook(),
    afterPlugins: new MiniSeriesHook(),
    invalid: new MiniSeriesHook(),
  }
}

export type CompilerHooks = ReturnType<typeof createCompilerHooks>
