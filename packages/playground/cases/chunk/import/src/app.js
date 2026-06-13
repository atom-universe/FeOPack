import txt from '../data.txt'
import fs from 'node:fs'
import path from 'node:path'

// esm
export const readFileData = () => {
    // const filePath = path.join(__dirname, '../data.txt')
    // const fileText = fs.readFileSync(filePath, 'utf-8')
    // console.log("fileText: ", txt)
    // console.log("lodashText: ", lodash.camelCase('hello lodash'))
    // console.log("typescriptMajor: ", ts.version.split('.')[0])
    return txt
}

export const writeFileData = () => {
    const filePath = path.join(__dirname, '../data.txt')
    fs.writeFileSync(filePath, 'Hello World!')
}
