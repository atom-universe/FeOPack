// const feopack = require("@feopack/core");

// /** @type { import('@feopack/core').FeopackOptions } */
module.exports = {
    context: __dirname,
    entry: "./src/index.js",
    mode: "development",
    output: {
      path: __dirname + "/dist",
      filename: "main.js"
    }
  };
  
  