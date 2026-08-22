import { describe, expect, it } from 'vitest'

import { formatBytes } from '@/utils/format'

describe('formatBytes', () => {
  it('formats zero as 0 B', () => {
    expect(formatBytes(0)).toBe('0 B')
  })

  it('formats negative or NaN values safely', () => {
    expect(formatBytes(-100)).toBe('0 B')
    expect(formatBytes(Number.NaN)).toBe('0 B')
  })

  it('formats raw bytes without decimals', () => {
    expect(formatBytes(512)).toBe('512 B')
  })

  it('formats kilobytes with default precision', () => {
    expect(formatBytes(1024)).toBe('1.00 KB')
    expect(formatBytes(1536)).toBe('1.50 KB')
  })

  it('formats megabytes and gigabytes', () => {
    expect(formatBytes(1024 ** 2)).toBe('1.00 MB')
    expect(formatBytes(8.6 * 1024 ** 3)).toBe('8.60 GB')
  })

  it('honours a custom fraction digit count', () => {
    expect(formatBytes(1024, 1)).toBe('1.0 KB')
    expect(formatBytes(2.5 * 1024 ** 2, 0)).toBe('3 MB')
  })
})