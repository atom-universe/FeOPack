/**
 * JS loader 执行阶段（对齐 webpack loader-runner / Rspack JsLoaderState）
 */
export enum JsLoaderState {
  Pitching = 'pitching',
  Normal = 'normal',
}

/**
 * Rust → Node 传递的上下文（对齐 Rspack `@rspack/binding` 的 `JsLoaderContext` 子集）。
 * 不传函数，只传数据；Node 侧据此构造 webpack 风格 `LoaderContext`。
 */
export interface JsLoaderContext {
  loaderState: JsLoaderState
  /** 已 resolve 的 loader 请求，如 `babel-loader?{...}` */
  loaders: string[]
  /** 合成 resource：`path + query + fragment` */
  resource: string
  /** pitch 阶段通常为空；normal 阶段为链上一步输出 */
  source: string
  /** 对应 webpack 配置里的 `context` 字段，
   * 个人认为叫 context 的话太容易和 loaderContext 搞混了
   */
  projectRoot: string
  cacheable?: boolean
  fileDependencies?: string[]
  contextDependencies?: string[]
  missingDependencies?: string[]
  /** 是否跳过读盘（Rust pitch 短路后由 Rust 传入 source） */
  skipReadResource?: boolean
}

/** @deprecated 使用 JsLoaderContext */
export type JsLoaderSnapshot = JsLoaderContext

export interface JsLoaderItem {
  path: string | null
  query: string | null
  fragment: string | null
  options: unknown
  ident: string | null
  normal: LoaderFn | null
  pitch: LoaderFn | null
  raw: boolean | null
  data: Record<string, unknown> | null
  pitchExecuted: boolean
  normalExecuted: boolean
  request: string
  type?: 'module' | 'commonjs'
}

export type LoaderFn = (
  this: LoaderContext,
  ...args: unknown[]
) => unknown

export interface LoaderContext {
  context: string | null
  loaderIndex: number
  loaders: JsLoaderItem[]
  resourcePath: string
  resourceQuery: string
  resourceFragment: string
  resource: string
  request: string
  remainingRequest: string
  currentRequest: string
  previousRequest: string
  query: unknown
  data: Record<string, unknown>
  async: (() => LoaderCallback) | null
  callback: LoaderCallback | null
  cacheable(flag?: boolean): void
  addDependency(file: string): void
  addContextDependency(context: string): void
  addMissingDependency(context: string): void
  getDependencies(): string[]
  getContextDependencies(): string[]
  getMissingDependencies(): string[]
  clearDependencies(): void
}

export type LoaderCallback = (
  err?: Error | null,
  content?: unknown,
  sourceMap?: unknown,
  meta?: unknown,
) => void

export interface RunLoadersOptions {
  resource?: string
  loaders?: string[]
  initialArgs?: unknown[]
  context?: Partial<LoaderContext>
  readResource?: (
    path: string,
    callback: (err: NodeJS.ErrnoException | null, buffer?: Buffer) => void,
  ) => void
  processResource?: (
    context: LoaderContext,
    resourcePath: string,
    callback: LoaderCallback,
  ) => void
}

export interface RunLoadersResult {
  result: unknown
  resourceBuffer: Buffer | null
  cacheable: boolean
  fileDependencies: string[]
  contextDependencies: string[]
  missingDependencies: string[]
}

export interface JsLoaderResult {
  source: string
  shortCircuit: boolean
  pitchedLoaderIndex?: number
  cacheable: boolean
  fileDependencies: string[]
  contextDependencies: string[]
  missingDependencies: string[]
}
