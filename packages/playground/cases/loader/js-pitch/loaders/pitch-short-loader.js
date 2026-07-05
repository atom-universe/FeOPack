'use strict'

// pitch 阶段直接 return，短路下一个 loader
module.exports.pitch = function pitchShortLoader() {
  return 'from js pitch'
}
