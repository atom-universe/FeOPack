import fs from 'node:fs'
import path from 'node:path'

// esm


export const writeFileData = () => {
    const filePath = path.join(__dirname, '../data.txt')
    fs.writeFileSync(filePath, 'Hello World!')
}
