const path = require('path')

/** @type { import('@feopack/core').FeopackOptions } */
module.exports = {
  context: __dirname,
  entry: './src/index.js',
  mode: 'development',
  module: {
    rules: [
      {
        test: '.demo',
        use: [path.resolve(__dirname, 'loaders/upper-loader.js')],
      },
    ],
  },
  output: {
    path: __dirname + '/dist',
    filename: 'main.js',
  },
}
