import { parseIdentifier } from './parse-identifier'
import type { JsLoaderItem } from './types'

class LoaderLoadingError extends Error {
  constructor(message: string) {
    super(message)
    this.name = 'LoaderLoadingError'
  }
}

function handleResult(
  loader: JsLoaderItem,
  module: unknown,
  callback: (err?: Error | null) => void,
): void {
  if (typeof module !== 'function' && (typeof module !== 'object' || module === null)) {
    callback(
      new LoaderLoadingError(
        `Module '${loader.path}' is not a loader (export function or es6 module)`,
      ),
    )
    return
  }

  const mod = module as {
    default?: unknown
    pitch?: unknown
    raw?: boolean
  }

  loader.normal =
    typeof module === 'function'
      ? (module as JsLoaderItem['normal'])
      : (mod.default as JsLoaderItem['normal'])
  loader.pitch = mod.pitch as JsLoaderItem['pitch']
  loader.raw = mod.raw ?? null

  if (typeof loader.normal !== 'function' && typeof loader.pitch !== 'function') {
    callback(
      new LoaderLoadingError(
        `Module '${loader.path}' is not a loader (must have normal or pitch function)`,
      ),
    )
    return
  }

  callback()
}

export function loadLoader(
  loader: JsLoaderItem,
  callback: (err?: Error | null) => void,
): void {
  if (!loader.path) {
    callback(new LoaderLoadingError('Loader path is empty'))
    return
  }

  if (loader.type === 'module') {
    import(loader.path)
      .then((module) => {
        handleResult(loader, module, callback)
      })
      .catch(callback)
    return
  }

  try {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const loadedModule = require(loader.path)
    handleResult(loader, loadedModule, callback)
  } catch (err) {
    const error = err as NodeJS.ErrnoException
    if (error.code === 'EMFILE') {
      setImmediate(() => loadLoader(loader, callback))
      return
    }
    callback(error)
  }
}

export function createLoaderObject(loader: string): JsLoaderItem {
  const obj: JsLoaderItem = {
    path: null,
    query: null,
    fragment: null,
    options: null,
    ident: null,
    normal: null,
    pitch: null,
    raw: null,
    data: null,
    pitchExecuted: false,
    normalExecuted: false,
    request: loader,
  }

  Object.defineProperty(obj, 'request', {
    enumerable: true,
    get() {
      return escapeHashLocal(obj.path) + escapeHashLocal(obj.query) + (obj.fragment || '')
    },
    set(value: string | { loader: string; options?: unknown; ident?: string; fragment?: string; type?: 'module' | 'commonjs' }) {
      if (typeof value === 'string') {
        const [path, query, fragment] = parseIdentifier(value)
        obj.path = path
        obj.query = query
        obj.fragment = fragment
        obj.options = undefined
        obj.ident = null
        return
      }

      if (!value.loader) {
        throw new Error(
          `request should be a string or object with loader and options (${JSON.stringify(value)})`,
        )
      }

      const { loader: path, fragment, type, options, ident } = value
      obj.path = path
      obj.fragment = fragment || ''
      obj.type = type
      obj.options = options
      obj.ident = ident ?? null

      if (options === null || options === undefined) {
        obj.query = ''
      } else if (typeof options === 'string') {
        obj.query = `?${options}`
      } else if (ident) {
        obj.query = `??${ident}`
      } else if (typeof options === 'object' && options && 'ident' in options) {
        obj.query = `??${(options as { ident: string }).ident}`
      } else {
        obj.query = `?${JSON.stringify(options)}`
      }
    },
  })

  obj.request = loader
  return obj
}

function escapeHashLocal(value: string | null): string {
  if (!value) return ''
  return value.includes('#') ? value.replace(/#/g, '\0#') : value
}
