'use strict'

/**
 * playground JS loader：读出 .demo 原文，转成 default export 模块
 */
module.exports = function upperLoader(source) {
  // 和 rspack 里面的设计一样，需要 context 的时候，可以从 this 上拿
  
  console.log('js loader context', this)
  const text = typeof source === 'string' ? source : String(source)
  const value = text.toUpperCase()
  return `__feopack_import__.d(__feopack_exports__, { default: () => ${JSON.stringify(value)} });`
}

// 从 this 上可以获取的：
// {
//   context: '/Users/carbon/Desktop/projects/opensource/bundlers/feopack/packages/playground/cases/loader/js-upper',
//   loaderIndex: 0,
//   loaders: [
//     {
//       path: '/Users/carbon/Desktop/projects/opensource/bundlers/feopack/packages/playground/cases/loader/js-upper/loaders/upper-loader.js',
//       query: '',
//       fragment: '',
//       options: undefined,
//       ident: null,
//       normal: [Function: upperLoader],
//       pitch: undefined,
//       raw: null,
//       data: null,
//       pitchExecuted: true,
//       normalExecuted: true,
//       request: [Getter/Setter]
//     }
//   ],
//   resourcePath: '/Users/carbon/Desktop/projects/opensource/bundlers/feopack/packages/playground/cases/loader/js-upper/src/data.demo',
//   resourceQuery: '',
//   resourceFragment: '',
//   async: [Function (anonymous)],
//   callback: [Function: innerCallback],
//   cacheable: [Function: cacheable],
//   addDependency: [Function: addDependency],
//   addContextDependency: [Function: addContextDependency],
//   addMissingDependency: [Function: addMissingDependency],
//   getDependencies: [Function: getDependencies],
//   getContextDependencies: [Function: getContextDependencies],
//   getMissingDependencies: [Function: getMissingDependencies],
//   clearDependencies: [Function: clearDependencies],
//   resource: [Getter/Setter],
//   request: [Getter],
//   remainingRequest: [Getter],
//   currentRequest: [Getter],
//   previousRequest: [Getter],
//   query: [Getter],
//   data: [Getter]
// }