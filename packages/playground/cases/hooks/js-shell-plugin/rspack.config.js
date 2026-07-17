const path = require('node:path')
const WebpackShellPluginNext = require('webpack-shell-plugin-next')

const logPath = path.join(__dirname, 'dist', 'js-plugin.log')
const appendLogScript = path.join(__dirname, 'append-log.js')

function appendLog(eventName) {
  return {
    command: process.execPath,
    args: [appendLogScript, logPath, eventName],
  }
}

module.exports = {
  entry: './src/index.js',
  plugins: [
    new WebpackShellPluginNext({
      logging: false,
      dev: false,
      onBeforeNormalRun: {
        scripts: [appendLog('beforeRun')],
        blocking: true,
      },
      onBeforeCompile: {
        scripts: [appendLog('beforeCompile')],
        blocking: true,
      },
      onBeforeBuild: {
        scripts: [appendLog('make')],
        blocking: true,
      },
      onBuildStart: {
        scripts: [appendLog('compilation')],
        blocking: true,
      },
      onBuildEnd: {
        scripts: [appendLog('afterEmit')],
        blocking: true,
      },
      onBuildExit: {
        scripts: [appendLog('done')],
        blocking: true,
      },
    }),
  ],
}
