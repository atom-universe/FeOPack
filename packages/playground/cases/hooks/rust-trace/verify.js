const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert');

module.exports = function verify(config) {
  const logPath = path.join(config.output.path, 'feopack-hooks.log');
  const lines = fs.readFileSync(logPath, 'utf8').trim().split(/\r?\n/);

  assert.deepStrictEqual(lines, [
    'before_run',
    'emit',
    'asset_emitted main.js',
    'done',
  ]);
};
