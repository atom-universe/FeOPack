import assert from 'node:assert/strict'
import path from 'node:path'
import test from 'node:test'
import { runLoadersAsync } from './index'
import { runJsLoaders } from './snapshot'
import { JsLoaderState } from './types'

const upperLoaderPath = path.join(__dirname, 'fixtures/upper-loader.js')

test('runLoaders 执行单个 JS loader', async () => {
  const result = await runLoadersAsync({
    resource: '/project/src/app.txt',
    loaders: [upperLoaderPath],
    processResource: (_ctx, _resourcePath, callback) => {
      callback(null, 'hello feopack')
    },
  })

  const content = Array.isArray(result.result) ? result.result[0] : result.result
  assert.equal(content, 'HELLO FEOPACK')
})

test('runJsLoaders 从 JsLoaderContext 跑 JS loader（Rspack 风格）', async () => {
  const result = await runJsLoaders({
    loaderState: JsLoaderState.Normal,
    loaders: [upperLoaderPath],
    resource: '/project/src/app.txt',
    source: 'hello snapshot',
    context: '/project',
    skipReadResource: true,
  })

  assert.equal(result.source, 'HELLO SNAPSHOT')
})
