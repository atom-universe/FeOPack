const fs = require('node:fs')
const path = require('node:path')

const logPath = path.join(__dirname, 'dist', 'watch-hooks.log')

function appendLog(event) {
  fs.mkdirSync(path.dirname(logPath), { recursive: true })
  fs.appendFileSync(logPath, `${event}\n`)
}

module.exports = {
  entry: './src/index.js',
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.watchRun.tap('WatchTracePlugin', () => {
          appendLog('watchRun')
        })
        compiler.hooks.invalid.tap('WatchTracePlugin', () => {
          appendLog('invalid')
        })
        compiler.hooks.watchClose.tap('WatchTracePlugin', () => {
          appendLog('watchClose')
        })
      },
    },
  ],
}
