import { Compiler } from './Compiler'

export type FeopackPlugin =
  | ((compiler: Compiler) => void)
  | {
      apply(compiler: Compiler): void
    }

export interface FeopackOptions {
  context: string
  entry: string | string[] | Record<string, string | string[]>
  mode?: 'development' | 'production'
  output: {
    path: string
    filename: string
  }
  module?: {
    rules?: Array<{
      test: string | RegExp
      use: string | string[]
    }>
  }
  rustPlugins?: string[]
  plugins?: FeopackPlugin[]
}

export { Watching } from './Watching'
export type { WatchHandler, WatchOptions } from './Watching'

export {
  runLoaders,
  runLoadersAsync,
  getContext,
} from './loader-runner'
export {
  runJsLoaders,
  jsLoaderContextToRunLoadersOptions,
  runJsLoadersFromSnapshot,
  snapshotToRunLoadersOptions,
} from './loader-runner/snapshot'
export {
  registerJsLoaderDispatcher,
  invokeJsLoaderDispatcher,
  createDefaultJsLoaderDispatcher,
} from './loader-runner/dispatcher'
export type {
  JsLoaderContext,
  JsLoaderSnapshot,
  JsLoaderResult,
  JsLoaderState,
  LoaderContext,
  RunLoadersOptions,
  RunLoadersResult,
} from './loader-runner/types'

function feopack(options: FeopackOptions): Compiler {
  return new Compiler(options)
}

export default feopack
