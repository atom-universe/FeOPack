'use strict'

/**
 * 这个 loader 是一个顺序探针，想验证的是：
 * 当一条 loader chain 同时包含 Rust loader 和 JS loader 时，
 * feopack 现在到底是不是按照同一条原始 chain 的语义去执行
 * （之前实现的一个版本中，是拆分成两个 chain 执行的，先跑 rust chain，再跑 js chain）
 * 这个 case 可以从最终产物直接观察到
 * 当前 mixed rust js loader chain 是否保持了预期的执行语义
 */
module.exports = function markerLoader(source) {
  const text = typeof source === 'string' ? source : String(source)

  if (text.includes('const __feopack_text__ = ')) {
    return text.replace(
      'const __feopack_text__ = ',
      'const __feopack_text__ = "[js-after-rust]" + ',
    )
  }

  return `${text.trimEnd()}\n[js-before-rust]`
}
