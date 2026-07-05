import { FeopackOptions } from '..'

function normalizeTest(test: string | RegExp): string {
  if (typeof test === 'string') {
    return test.startsWith('.') ? test : `.${test}`
  }
  const source = test.source
  if (source.startsWith('\\.')) {
    return `.${source.slice(2).replace(/\\\./g, '.')}`
  }
  return `.${source}`
}

/**
 * 转化为 rust 理解的格式
 */
export const getRawOptions = (options: FeopackOptions) => {
  const moduleRules =
    options.module?.rules?.map((rule) => {
      const useLoaders = Array.isArray(rule.use) ? rule.use : [rule.use]
      return {
        test: normalizeTest(rule.test as string | RegExp),
        useLoaders: useLoaders.map((loader) =>
          typeof loader === 'string' ? loader : loader,
        ),
      }
    }) ?? []

  return {
    context: options.context,
    entry: typeof options.entry === 'string' ? options.entry : Object.values(options.entry)[0] as string,
    mode: options.mode || 'production',
    output: options.output,
    module: moduleRules.length > 0 ? { rules: moduleRules } : undefined,
  }
}
