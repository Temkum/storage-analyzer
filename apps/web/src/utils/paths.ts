export function normalizePath(path: string): string {
  return path.replace(/\\/g, '/').replace(/\/+$/, '')
}

export function isDirectChild(path: string, root: string): boolean {
  const normalizedPath = normalizePath(path)
  const normalizedRoot = normalizePath(root)

  if (normalizedPath === normalizedRoot) {
    return false
  }

  const relative = normalizedPath.startsWith(`${normalizedRoot}/`)
    ? normalizedPath.slice(normalizedRoot.length + 1)
    : ''

  return relative.length > 0 && !relative.includes('/')
}

export function basename(path: string): string {
  const normalized = normalizePath(path)
  const parts = normalized.split('/')

  return parts[parts.length - 1] || normalized
}

export function relativePath(path: string, root: string): string {
  const normalized = normalizePath(path)
  const normalizedRoot = normalizePath(root)

  if (normalized === normalizedRoot) {
    return normalized
  }

  if (normalized.startsWith(`${normalizedRoot}/`)) {
    return normalized.slice(normalizedRoot.length + 1)
  }

  return normalized
}
