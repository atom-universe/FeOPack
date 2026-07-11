import { runLoadersNormalOnlyAsync, runLoadersPitchOnlyAsync } from './index'
import type { JsLoaderContext, JsLoaderResult, RunLoadersOptions } from './types'
import { JsLoaderState } from './types'

/**
 * 把 Rspack 风格的 JsLoaderContext 转成 fork 的 loader-runner options。
 */
export function jsLoaderContextToRunLoadersOptions(
  ctx: JsLoaderContext,
): RunLoadersOptions {
  return {
    resource: ctx.resource,
    loaders: ctx.loaders,
    context: {
      context: ctx.projectRoot,
    },
    processResource: (loaderContext, resourcePath, callback) => {
      loaderContext.addDependency(resourcePath)

      if (ctx.skipReadResource || ctx.source) {
        callback(null, ctx.source)
        return
      }

      callback(new Error('feopack: JS loader-runner 需要 source 或 skipReadResource'))
    },
  }
}

function normalizeRunResult(result: unknown): string {
  if (result === null || result === undefined) {
    return ''
  }

  if (Array.isArray(result)) {
    const [content] = result
    if (typeof content === 'string') {
      return content
    }
    if (Buffer.isBuffer(content)) {
      return content.toString('utf8')
    }
  }

  if (typeof result === 'string') {
    return result
  }

  if (Buffer.isBuffer(result)) {
    return result.toString('utf8')
  }

  throw new Error(`feopack: 无法把 loader 结果转为 source: ${typeof result}`)
}

/**
 * 对齐 Rspack `runLoaders(compiler, context)` 的 feopack 入口。
 * Rust yield 到 Node 时调用。
 */
export async function runJsLoaders(ctx: JsLoaderContext): Promise<JsLoaderResult> {
  if (ctx.loaderState === JsLoaderState.Pitching) {
    const options = jsLoaderContextToRunLoadersOptions(ctx)
    const runResult = await runLoadersPitchOnlyAsync(options)
    return {
      source: normalizeRunResult(runResult.result),
      shortCircuit: runResult.kind === 'shortCircuit',
      pitchedLoaderIndex: runResult.pitchedLoaderIndex,
      cacheable: runResult.cacheable,
      fileDependencies: runResult.fileDependencies,
      contextDependencies: runResult.contextDependencies,
      missingDependencies: runResult.missingDependencies,
    }
  }

  const options = {
    ...jsLoaderContextToRunLoadersOptions(ctx),
    initialArgs: [ctx.source],
  }
  const runResult = await runLoadersNormalOnlyAsync(options)

  return {
    source: normalizeRunResult(runResult.result),
    shortCircuit: false,
    cacheable: runResult.cacheable,
    fileDependencies: runResult.fileDependencies,
    contextDependencies: runResult.contextDependencies,
    missingDependencies: runResult.missingDependencies,
  }
}

/** @deprecated 使用 runJsLoaders */
export const runJsLoadersFromSnapshot = runJsLoaders

/** @deprecated 使用 jsLoaderContextToRunLoadersOptions */
export const snapshotToRunLoadersOptions = jsLoaderContextToRunLoadersOptions
