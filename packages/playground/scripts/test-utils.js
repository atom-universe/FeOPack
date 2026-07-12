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

function normalizeConfig(config, parent) {
  return {
    ...config,
    context: config.context || parent,
    output: {
      path: path.join(parent, 'dist'),
      filename: 'main.js',
      ...config.output,
    },
  };
}

function executeDist(config) {
  if (config.__skipExecute) {
    return;
  }

  const outputPath = config.output.path || path.join(config.context, 'dist');
  const filename = config.output.filename || 'main.js';
  const bundlePath = path.join(outputPath, filename);

  if (!fs.existsSync(bundlePath)) {
    throw new Error(`Dist file not found: ${bundlePath}`);
  }

  delete require.cache[require.resolve(bundlePath)];
  console.log();
  console.log('='.repeat(10), ' 产物运行结果 ', '='.repeat(10));
  require(bundlePath);
  console.log('='.repeat(35), '\n');
}

const CONFIG_FILENAME = 'rspack.config.js';

async function run(filter = []) {
  const rootPath = path.join(__dirname, '../cases');
  // TODO: 更完美的版本可以支持中断
  // TODO: 支持遍历模式设置 -- DFS、BFS、甚至更多更加复杂的遍历

  const tasks = [];

  walk(rootPath, ({ fullPath, dirent }) => {
    if(fullPath.includes('meow')) {
      console.log('什么玩意儿', fullPath)
    }

    if (dirent.name === CONFIG_FILENAME) {
      // console.log('fullPath', fullPath);
      const parent = path.dirname(fullPath);
      const casePath = path.relative(rootPath, parent);

      if (!filter.length || filter.includes(casePath)) {
        tasks.push(async () => {
          delete require.cache[require.resolve(fullPath)];
          const config = normalizeConfig(require(fullPath), parent);
          fs.rmSync(config.output.path, { recursive: true, force: true });
          await feopack(config).run();
          executeDist(config);

          const verifyPath = path.join(parent, 'verify.js');
          if (fs.existsSync(verifyPath)) {
            delete require.cache[require.resolve(verifyPath)];
            require(verifyPath)(config);
          }
        });
      }
    }
  });

  for (const task of tasks) {
    await task();
  }
};


module.exports = {
  walk,
  run,
};
