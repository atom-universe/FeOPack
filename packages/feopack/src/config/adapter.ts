import { FeopackOptions } from '..'

/**
 * 转化为 rust 理解的格式
 */
export const getRawOptions = (options: FeopackOptions) => {
  return {
    context: options.context,
    entry: options.entry,
    mode: options.mode || 'production',
    output: options.output,
  }
}
