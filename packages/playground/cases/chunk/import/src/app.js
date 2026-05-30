import fs from 'node:fs'
import path from 'node:path'
import ts from 'typescript'

const lodash = require('lodash')

export const readFileData = () => {
    const filePath = path.join(__dirname, '../data.txt')
    const fileText = fs.readFileSync(filePath, 'utf-8')
    console.log("fileText: ", fileText)
    console.log("lodashText: ", lodash.camelCase('hello lodash'))
    console.log("typescriptMajor: ", ts.version.split('.')[0])
}
