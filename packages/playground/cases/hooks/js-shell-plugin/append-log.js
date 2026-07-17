const fs = require('node:fs')
const path = require('node:path')

const logPath = process.argv[2]
const eventName = process.argv[3]

fs.mkdirSync(path.dirname(logPath), { recursive: true })
fs.appendFileSync(logPath, `${eventName}\n`)
