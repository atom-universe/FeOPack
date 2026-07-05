const HASH_ESCAPE_REGEXP = /#/g
const PATH_QUERY_FRAGMENT_REGEXP =
  /^((?:\0.|[^?#\0])*)(\?(?:\0.|[^#\0])*)?(#.*)?$/
const ZERO_ESCAPE_REGEXP = /\0(.)/g

export function escapeHash(str: string): string {
  return str.includes('#') ? str.replace(HASH_ESCAPE_REGEXP, '\0#') : str
}

export function parseIdentifier(identifier: string): [string, string, string] {
  const firstEscape = identifier.indexOf('\0')

  if (firstEscape < 0) {
    const queryStart = identifier.indexOf('?')
    const fragmentStart = identifier.indexOf('#')

    if (fragmentStart < 0) {
      if (queryStart < 0) {
        return [identifier, '', '']
      }
      return [identifier.slice(0, queryStart), identifier.slice(queryStart), '']
    }

    if (queryStart < 0 || fragmentStart < queryStart) {
      return [identifier.slice(0, fragmentStart), '', identifier.slice(fragmentStart)]
    }

    return [
      identifier.slice(0, queryStart),
      identifier.slice(queryStart, fragmentStart),
      identifier.slice(fragmentStart),
    ]
  }

  const match = PATH_QUERY_FRAGMENT_REGEXP.exec(identifier)
  if (!match) {
    return [identifier, '', '']
  }

  return [
    match[1].replace(ZERO_ESCAPE_REGEXP, '$1'),
    match[2] ? match[2].replace(ZERO_ESCAPE_REGEXP, '$1') : '',
    match[3] || '',
  ]
}

export function dirname(path: string): string {
  if (path === '/') return '/'
  const i = path.lastIndexOf('/')
  const j = path.lastIndexOf('\\')
  const i2 = path.indexOf('/')
  const j2 = path.indexOf('\\')
  const idx = i > j ? i : j
  const idx2 = i > j ? i2 : j2
  if (idx < 0) return path
  if (idx === idx2) return path.slice(0, idx + 1)
  return path.slice(0, idx)
}

export function joinRequests(
  loaders: { request: string }[],
  start: number,
  end: number,
  resource: string,
): string {
  let result = ''
  for (let i = start; i < end; i++) {
    result += `${loaders[i].request}!`
  }
  return result + resource
}
