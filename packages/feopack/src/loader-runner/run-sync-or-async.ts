import type { LoaderCallback, LoaderContext } from './types'

const UTF8_BOM_0 = 0xef
const UTF8_BOM_1 = 0xbb
const UTF8_BOM_2 = 0xbf

function utf8BufferToString(buf: Buffer): string {
  if (
    buf.length >= 3 &&
    buf[0] === UTF8_BOM_0 &&
    buf[1] === UTF8_BOM_1 &&
    buf[2] === UTF8_BOM_2
  ) {
    return buf.toString('utf8', 3)
  }
  return buf.toString('utf8')
}

export function runSyncOrAsync(
  fn: (...args: unknown[]) => unknown,
  context: LoaderContext,
  args: unknown[],
  callback: LoaderCallback,
): void {
  let isSync = true
  let isDone = false
  let isError = false
  let reportedError = false

  const innerCallback: LoaderCallback = (...callbackArgs) => {
    if (isDone) {
      if (reportedError) return
      throw new Error('callback(): The callback was already called.')
    }

    isDone = true
    isSync = false

    try {
      callback(...callbackArgs)
    } catch (err) {
      isError = true
      throw err
    }
  }

  context.callback = innerCallback
  context.async = () => {
    if (isDone) {
      if (reportedError) return () => innerCallback
      throw new Error('async(): The callback was already called.')
    }
    isSync = false
    return innerCallback
  }

  try {
    const result = fn.apply(context, args)
    if (isSync) {
      isDone = true
      if (result === undefined) {
        callback()
        return
      }
      if (
        result &&
        typeof result === 'object' &&
        'then' in result &&
        typeof (result as Promise<unknown>).then === 'function'
      ) {
        ;(result as Promise<unknown>).then(
          (r) => callback(null, r),
          (err) => callback(err as Error),
        )
        return
      }
      callback(null, result)
    }
  } catch (err) {
    if (isError) throw err
    if (isDone) {
      console.error(err)
      return
    }
    isDone = true
    reportedError = true
    callback(err as Error)
  }
}

export function convertArgs(args: unknown[], raw: boolean | null): void {
  if (!args.length) return
  if (!raw && Buffer.isBuffer(args[0])) {
    args[0] = utf8BufferToString(args[0] as Buffer)
  } else if (raw && typeof args[0] === 'string') {
    args[0] = Buffer.from(args[0] as string, 'utf8')
  }
}
