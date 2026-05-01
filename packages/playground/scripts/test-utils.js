const fs = require("node:fs");
const path = require("node:path");
const feopack = require("@feopack/core").default;

function walk(root, callback) {
  const entries = fs.readdirSync(root, { withFileTypes: true });
  for (const dirent of entries) {
    const fullPath = path.join(root, dirent.name);
    const ret = callback({ fullPath, dirent, parent: root });
    if (dirent.isDirectory()) {
      if (ret === false) continue; // 允许外部决定跳过
      walk(fullPath, callback);
    }
  }
}

const CONFIG_FILENAME = 'rspack.config.js';

function run(filter = []) {
  const rootPath = path.join(__dirname, '../cases');
  // TODO: 更完美的版本可以支持中断
  // TODO: 支持遍历模式设置 -- DFS、BFS、甚至更多更加复杂的遍历

  walk(rootPath, ({ fullPath, dirent }) => {
    if (dirent.name === CONFIG_FILENAME) {
      // console.log('fullPath', fullPath);
      const parent = path.dirname(fullPath);

      if (!filter.length || filter.some(f => parent.endsWith(f))) {
        const config = require(fullPath);
        feopack(config).run();
      }
    }
  });
};


module.exports = {
  walk,
  run,
};
