// pnpm test
// pnpm test basic
const { run } = require("./test-utils");


const filteredPkg = process.argv.slice(2);

console.log('测试目标', filteredPkg);

run(filteredPkg).catch(error => {
  console.error(error);
  process.exit(1);
});

