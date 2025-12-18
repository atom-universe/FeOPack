import { Compiler } from './Compiler'

export interface FeopackOptions {
  context: string
  entry: string | string[] | Record<string, string | string[]>
  output: {
    path: string
    filename: string
  }
}

function feopack(options: FeopackOptions): Compiler {
  return new Compiler(options)
}

export default feopack
