import fs from 'node:fs'
import path from 'node:path'
import ts from 'typescript'
import lodash from 'lodash'

// esm
export const readFileData = () => {
    const filePath = path.join(__dirname, '../data.txt')
    const fileText = fs.readFileSync(filePath, 'utf-8')
    console.log("fileText: ", fileText)
    console.log("lodashText: ", lodash.camelCase('hello lodash'))
    console.log("typescriptMajor: ", ts.version.split('.')[0])
}

export const writeFileData = () => {
    const filePath = path.join(__dirname, '../data.txt')
    fs.writeFileSync(filePath, 'Hello World!')
}
