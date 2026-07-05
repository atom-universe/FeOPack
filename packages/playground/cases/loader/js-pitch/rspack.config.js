const path = require('path')

/** @type { import('@feopack/core').FeopackOptions } */
module.exports = {
  context: __dirname,
  entry: './src/index.js',
  mode: 'development',
  module: {
    rules: [
      {
        test: '.txt',
        use: ['text-loader', path.resolve(__dirname, 'loaders/pitch-short-loader.js')],
      },
    ],
  },
  output: {
    path: __dirname + '/dist',
    filename: 'main.js',
  },
}
