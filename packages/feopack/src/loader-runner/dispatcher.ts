import { runJsLoaders } from './snapshot'
import type { JsLoaderContext, JsLoaderResult } from './types'

export type JsLoaderDispatcher = (context: JsLoaderContext) => Promise<JsLoaderResult>

let dispatcher: JsLoaderDispatcher | null = null

/**
 * 在 Compiler 初始化时注册，供 napi 从 Rust 回调 Node（对齐 Rspack dispatcher 角色）。
 */
export function registerJsLoaderDispatcher(fn: JsLoaderDispatcher): void {
  dispatcher = fn
}

export function getJsLoaderDispatcher(): JsLoaderDispatcher | null {
  return dispatcher
}

/**
 * Rust 通过 napi 调用时的统一入口。
 */
export async function invokeJsLoaderDispatcher(
  context: JsLoaderContext,
): Promise<JsLoaderResult> {
  if (!dispatcher) {
    return runJsLoaders(context)
  }
  return dispatcher(context)
}

export function createDefaultJsLoaderDispatcher(): JsLoaderDispatcher {
  return runJsLoaders
}
