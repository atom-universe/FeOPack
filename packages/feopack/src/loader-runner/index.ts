/**
 * fork 自 webpack/loader-runner，形态对齐 Rspack packages/rspack/src/loader-runner。
 * feopack 仅保留 pitch/normal 核心；完整版见 Rspack 仓库。
 */
import { readFile } from 'node:fs'
import { createLoaderObject, loadLoader } from './load-loader'
import { dirname, joinRequests, parseIdentifier } from './parse-identifier'
import { convertArgs, runSyncOrAsync } from './run-sync-or-async'
import type {
  JsLoaderItem,
  LoaderCallback,
  LoaderContext,
  RunLoadersOptions,
  RunLoadersResult,
} from './types'

export interface RunPitchLoadersResult {
  kind: 'continue' | 'shortCircuit'
  result?: unknown[]
  pitchedLoaderIndex?: number
  cacheable: boolean
  fileDependencies: string[]
  contextDependencies: string[]
  missingDependencies: string[]
}

interface ProcessOptions {
  resourceBuffer: Buffer | null
  processResource: (
    context: LoaderContext,
    resourcePath: string,
    callback: LoaderCallback,
  ) => void
}

function iterateNormalLoaders(
  options: ProcessOptions,
  loaderContext: LoaderContext,
  args: unknown[],
  callback: LoaderCallback,
): void {
  while (loaderContext.loaderIndex >= 0) {
    const currentLoaderObject = loaderContext.loaders[loaderContext.loaderIndex]

    if (currentLoaderObject.normalExecuted) {
      loaderContext.loaderIndex--
      continue
    }

    const fn = currentLoaderObject.normal
    currentLoaderObject.normalExecuted = true

    if (!fn) {
      loaderContext.loaderIndex--
      continue
    }

    convertArgs(args, currentLoaderObject.raw)

    runSyncOrAsync(fn, loaderContext, args, (err, ...nextArgs) => {
      if (err) {
        callback(err)
        return
      }
      iterateNormalLoaders(options, loaderContext, nextArgs, callback)
    })
    return
  }

  callback(null, args)
}

function iterateNormalLoadersOnly(
  loaderContext: LoaderContext,
  args: unknown[],
  callback: LoaderCallback,
): void {
  while (loaderContext.loaderIndex >= 0) {
    const currentLoaderObject = loaderContext.loaders[loaderContext.loaderIndex]

    if (currentLoaderObject.normalExecuted) {
      loaderContext.loaderIndex--
      continue
    }

    loadLoader(currentLoaderObject, (err) => {
      if (err) {
        callback(err)
        return
      }

      const fn = currentLoaderObject.normal
      currentLoaderObject.normalExecuted = true

      if (!fn) {
        loaderContext.loaderIndex--
        iterateNormalLoadersOnly(loaderContext, args, callback)
        return
      }

      convertArgs(args, currentLoaderObject.raw)

      runSyncOrAsync(fn, loaderContext, args, (runErr, ...nextArgs) => {
        if (runErr) {
          callback(runErr)
          return
        }
        iterateNormalLoadersOnly(loaderContext, nextArgs, callback)
      })
    })
    return
  }

  callback(null, args)
}

function processResource(
  options: ProcessOptions,
  loaderContext: LoaderContext,
  callback: LoaderCallback,
): void {
  loaderContext.loaderIndex = loaderContext.loaders.length - 1
  const { resourcePath } = loaderContext

  if (!resourcePath) {
    iterateNormalLoaders(options, loaderContext, [null], callback)
    return
  }

  options.processResource(loaderContext, resourcePath, (err, ...args) => {
    if (err) {
      callback(err)
      return
    }
    options.resourceBuffer = Buffer.isBuffer(args[0]) ? (args[0] as Buffer) : null
    iterateNormalLoaders(options, loaderContext, args, callback)
  })
}

function iteratePitchingLoaders(
  options: ProcessOptions,
  loaderContext: LoaderContext,
  callback: LoaderCallback,
): void {
  while (loaderContext.loaderIndex < loaderContext.loaders.length) {
    const currentLoaderObject = loaderContext.loaders[loaderContext.loaderIndex]

    if (currentLoaderObject.pitchExecuted) {
      loaderContext.loaderIndex++
      continue
    }

    loadLoader(currentLoaderObject, (err) => {
      if (err) {
        loaderContext.cacheable(false)
        callback(err)
        return
      }

      const fn = currentLoaderObject.pitch
      currentLoaderObject.pitchExecuted = true

      if (!fn) {
        loaderContext.loaderIndex++
        iteratePitchingLoaders(options, loaderContext, callback)
        return
      }

      runSyncOrAsync(
        fn,
        loaderContext,
        [
          loaderContext.remainingRequest,
          loaderContext.previousRequest,
          (currentLoaderObject.data = {}),
        ],
        (pitchErr, ...args) => {
          if (pitchErr) {
            callback(pitchErr)
            return
          }

          let hasArg = false
          for (let i = 0; i < args.length; i++) {
            if (args[i] !== undefined) {
              hasArg = true
              break
            }
          }

          if (hasArg) {
            loaderContext.loaderIndex--
            iterateNormalLoaders(options, loaderContext, args, callback)
          } else {
            loaderContext.loaderIndex++
            iteratePitchingLoaders(options, loaderContext, callback)
          }
        },
      )
    })
    return
  }

  processResource(options, loaderContext, callback)
}

function iteratePitchingLoadersOnly(
  loaderContext: LoaderContext,
  callback: (err: Error | null, result?: { kind: 'continue' | 'shortCircuit'; args?: unknown[]; pitchedLoaderIndex?: number }) => void,
): void {
  while (loaderContext.loaderIndex < loaderContext.loaders.length) {
    const currentLoaderObject = loaderContext.loaders[loaderContext.loaderIndex]

    if (currentLoaderObject.pitchExecuted) {
      loaderContext.loaderIndex++
      continue
    }

    loadLoader(currentLoaderObject, (err) => {
      if (err) {
        loaderContext.cacheable(false)
        callback(err)
        return
      }

      const fn = currentLoaderObject.pitch
      currentLoaderObject.pitchExecuted = true

      if (!fn) {
        loaderContext.loaderIndex++
        iteratePitchingLoadersOnly(loaderContext, callback)
        return
      }

      runSyncOrAsync(
        fn,
        loaderContext,
        [
          loaderContext.remainingRequest,
          loaderContext.previousRequest,
          (currentLoaderObject.data = {}),
        ],
        (pitchErr, ...args) => {
          if (pitchErr) {
            callback(pitchErr)
            return
          }

          let hasArg = false
          for (let i = 0; i < args.length; i++) {
            if (args[i] !== undefined) {
              hasArg = true
              break
            }
          }

          if (hasArg) {
            callback(null, {
              kind: 'shortCircuit',
              args,
              pitchedLoaderIndex: loaderContext.loaderIndex,
            })
            return
          }

          loaderContext.loaderIndex++
          iteratePitchingLoadersOnly(loaderContext, callback)
        },
      )
    })
    return
  }

  callback(null, { kind: 'continue' })
}

function buildLoaderContext(
  resource: string,
  loaders: JsLoaderItem[],
  extra?: Partial<LoaderContext>,
): { loaderContext: LoaderContext; getCacheable: () => boolean } {
  const splittedResource = resource ? parseIdentifier(resource) : null
  const resourcePath = splittedResource ? splittedResource[0] : ''
  const resourceQuery = splittedResource ? splittedResource[1] : ''
  const resourceFragment = splittedResource ? splittedResource[2] : ''
  const contextDirectory = resourcePath ? dirname(resourcePath) : null

  let requestCacheable = true
  const fileDependencies: string[] = []
  const contextDependencies: string[] = []
  const missingDependencies: string[] = []

  const loaderContext = {
    context: contextDirectory,
    loaderIndex: 0,
    loaders,
    resourcePath,
    resourceQuery,
    resourceFragment,
    async: null,
    callback: null,
    cacheable: (flag?: boolean) => {
      if (flag === false) requestCacheable = false
    },
    addDependency: (file: string) => {
      fileDependencies.push(file)
    },
    addContextDependency: (context: string) => {
      contextDependencies.push(context)
    },
    addMissingDependency: (context: string) => {
      missingDependencies.push(context)
    },
    getDependencies: () => fileDependencies.slice(),
    getContextDependencies: () => contextDependencies.slice(),
    getMissingDependencies: () => missingDependencies.slice(),
    clearDependencies: () => {
      fileDependencies.length = 0
      contextDependencies.length = 0
      missingDependencies.length = 0
      requestCacheable = true
    },
    resource: '',
    request: '',
    remainingRequest: '',
    currentRequest: '',
    previousRequest: '',
    query: undefined as unknown,
    data: {},
    ...extra,
  } as LoaderContext

  Object.defineProperty(loaderContext, 'resource', {
    enumerable: true,
    get() {
      return loaderContext.resourcePath + loaderContext.resourceQuery + loaderContext.resourceFragment
    },
    set(value: string) {
      const splitted = value ? parseIdentifier(value) : null
      loaderContext.resourcePath = splitted ? splitted[0] : ''
      loaderContext.resourceQuery = splitted ? splitted[1] : ''
      loaderContext.resourceFragment = splitted ? splitted[2] : ''
    },
  })

  Object.defineProperty(loaderContext, 'request', {
    enumerable: true,
    get() {
      return joinRequests(loaders, 0, loaders.length, loaderContext.resource || '')
    },
  })

  Object.defineProperty(loaderContext, 'remainingRequest', {
    enumerable: true,
    get() {
      return joinRequests(
        loaders,
        loaderContext.loaderIndex + 1,
        loaders.length,
        loaderContext.resource,
      )
    },
  })

  Object.defineProperty(loaderContext, 'currentRequest', {
    enumerable: true,
    get() {
      return joinRequests(
        loaders,
        loaderContext.loaderIndex,
        loaders.length,
        loaderContext.resource,
      )
    },
  })

  Object.defineProperty(loaderContext, 'previousRequest', {
    enumerable: true,
    get() {
      const end = loaderContext.loaderIndex
      if (end === 0) return ''
      let result = loaders[0].request
      for (let i = 1; i < end; i++) {
        result += `!${loaders[i].request}`
      }
      return result
    },
  })

  Object.defineProperty(loaderContext, 'query', {
    enumerable: true,
    get() {
      const entry = loaders[loaderContext.loaderIndex]
      return entry.options && typeof entry.options === 'object'
        ? entry.options
        : entry.query
    },
  })

  Object.defineProperty(loaderContext, 'data', {
    enumerable: true,
    get() {
      return loaders[loaderContext.loaderIndex].data ?? {}
    },
  })

  return {
    loaderContext,
    getCacheable: () => requestCacheable,
  }
}

export function getContext(resource: string): string {
  const [path] = parseIdentifier(resource)
  return dirname(path)
}

/**
 * fork 自 webpack/loader-runner 的 runLoaders。
 * feopack 在 Node 侧执行 JS loader 链的入口。
 */
export function runLoaders(
  options: RunLoadersOptions,
  callback: (err: Error | null, result?: RunLoadersResult) => void,
): void {
  const resource = options.resource || ''
  const readResource =
    options.readResource ||
    ((path, cb) => {
      readFile(path, cb)
    })

  const processResourceFn =
    options.processResource ||
    ((context, resourcePath, cb) => {
      context.addDependency(resourcePath)
      readResource(resourcePath, (err, buffer) => {
        if (err) {
          cb(err)
          return
        }
        cb(null, buffer)
      })
    })

  const loaders = (options.loaders || []).map(createLoaderObject)
  const { loaderContext, getCacheable } = buildLoaderContext(resource, loaders, options.context)

  const processOptions: ProcessOptions = {
    resourceBuffer: null,
    processResource: processResourceFn,
  }

  iteratePitchingLoaders(processOptions, loaderContext, (err, result) => {
    if (err) {
      callback(err, {
        result: undefined,
        resourceBuffer: processOptions.resourceBuffer,
        cacheable: false,
        fileDependencies: loaderContext.getDependencies(),
        contextDependencies: loaderContext.getContextDependencies(),
        missingDependencies: loaderContext.getMissingDependencies(),
      })
      return
    }

    callback(null, {
      result,
      resourceBuffer: processOptions.resourceBuffer,
      cacheable: getCacheable(),
      fileDependencies: loaderContext.getDependencies(),
      contextDependencies: loaderContext.getContextDependencies(),
      missingDependencies: loaderContext.getMissingDependencies(),
    })
  })
}

export function runLoadersPitchOnly(
  options: RunLoadersOptions,
  callback: (err: Error | null, result?: RunPitchLoadersResult) => void,
): void {
  const resource = options.resource || ''
  const loaders = (options.loaders || []).map(createLoaderObject)
  const { loaderContext, getCacheable } = buildLoaderContext(resource, loaders, options.context)

  iteratePitchingLoadersOnly(loaderContext, (err, result) => {
    if (err || !result) {
      callback(err ?? new Error('runLoadersPitchOnly failed without error'), {
        kind: 'continue',
        cacheable: false,
        fileDependencies: loaderContext.getDependencies(),
        contextDependencies: loaderContext.getContextDependencies(),
        missingDependencies: loaderContext.getMissingDependencies(),
      })
      return
    }

    callback(null, {
      kind: result.kind,
      result: result.args,
      pitchedLoaderIndex: result.pitchedLoaderIndex,
      cacheable: getCacheable(),
      fileDependencies: loaderContext.getDependencies(),
      contextDependencies: loaderContext.getContextDependencies(),
      missingDependencies: loaderContext.getMissingDependencies(),
    })
  })
}

export function runLoadersNormalOnly(
  options: RunLoadersOptions,
  callback: (err: Error | null, result?: RunLoadersResult) => void,
): void {
  const resource = options.resource || ''
  const loaders = (options.loaders || []).map(createLoaderObject)
  const { loaderContext, getCacheable } = buildLoaderContext(resource, loaders, options.context)
  const initialArgs = options.initialArgs ?? [null]

  loaderContext.loaderIndex = loaders.length - 1

  iterateNormalLoadersOnly(loaderContext, initialArgs, (err, result) => {
    if (err) {
      callback(err, {
        result: undefined,
        resourceBuffer: null,
        cacheable: false,
        fileDependencies: loaderContext.getDependencies(),
        contextDependencies: loaderContext.getContextDependencies(),
        missingDependencies: loaderContext.getMissingDependencies(),
      })
      return
    }

    callback(null, {
      result,
      resourceBuffer: null,
      cacheable: getCacheable(),
      fileDependencies: loaderContext.getDependencies(),
      contextDependencies: loaderContext.getContextDependencies(),
      missingDependencies: loaderContext.getMissingDependencies(),
    })
  })
}

export function runLoadersAsync(options: RunLoadersOptions): Promise<RunLoadersResult> {
  return new Promise((resolve, reject) => {
    runLoaders(options, (err, result) => {
      if (err || !result) {
        reject(err ?? new Error('runLoaders failed without error'))
        return
      }
      resolve(result)
    })
  })
}

export function runLoadersPitchOnlyAsync(
  options: RunLoadersOptions,
): Promise<RunPitchLoadersResult> {
  return new Promise((resolve, reject) => {
    runLoadersPitchOnly(options, (err, result) => {
      if (err || !result) {
        reject(err ?? new Error('runLoadersPitchOnly failed without error'))
        return
      }
      resolve(result)
    })
  })
}

export function runLoadersNormalOnlyAsync(
  options: RunLoadersOptions,
): Promise<RunLoadersResult> {
  return new Promise((resolve, reject) => {
    runLoadersNormalOnly(options, (err, result) => {
      if (err || !result) {
        reject(err ?? new Error('runLoadersNormalOnly failed without error'))
        return
      }
      resolve(result)
    })
  })
}
