const BYTE_UNITS = ['B', 'KB', 'MB', 'GB', 'TB'] as const

export function formatBytes(bytes: number, fractionDigits = 2): string {
  if (!Number.isFinite(bytes) || bytes <= 0) {
    return '0 B'
  }

  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), BYTE_UNITS.length - 1)

  const digits = index === 0 ? 0 : fractionDigits

  return `${(bytes / 1024 ** index).toFixed(digits)} ${BYTE_UNITS[index]}`
}

/** Formats a per-second byte rate, e.g. `12.4 MB/s`. */
export function formatRate(bytesPerSecond: number, fractionDigits = 1): string {
  return `${formatBytes(bytesPerSecond, fractionDigits)}/s`
}
