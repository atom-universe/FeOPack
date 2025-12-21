#!/usr/bin/env node
import { resolve } from 'path'
import { existsSync } from 'fs'
import feopack from '@feopack/core'

// 解析命令行参数
const args = process.argv.slice(2)
let configPath = 'rspack.config.js'

// 解析 -c 参数
if (args.length > 0) {
  if (args[0] === '-c' || args[0] === '--config') {
    configPath = args[1] || 'rspack.config.js'
  } else {
    // 如果没有 -c，第一个参数就是配置文件路径
    configPath = args[0]
  }
}

const configFile = resolve(process.cwd(), configPath)

if (!existsSync(configFile)) {
  console.error(`Error: Config file not found: ${configFile}`)
  console.error(`Current working directory: ${process.cwd()}`)
  process.exit(1)
}

// 加载配置文件（CommonJS）
// eslint-disable-next-line @typescript-eslint/no-require-imports
const config = require(configFile)

// 创建编译器并执行
try {
  console.log(`Building with config: ${configPath}`)
  const compiler = feopack(config)
  const compilation = compiler.run()

  console.log('Build completed!')
  console.log('Compilation', compilation)
} catch (error) {
  console.error('Build failed:', error)
  process.exit(1)
}
