'use strict'

/**
 * 测试用 JS loader：把 source 转成大写
 */
module.exports = function upperLoader(source) {
  if (typeof source !== 'string') {
    return source
  }
  return source.toUpperCase()
}
