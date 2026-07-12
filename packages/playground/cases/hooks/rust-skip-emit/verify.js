const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert');

module.exports = function verify(config) {
  const bundlePath = path.join(config.output.path, config.output.filename);

  assert.strictEqual(fs.existsSync(bundlePath), false);
};
